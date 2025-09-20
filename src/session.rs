use crate::core::{Cookie, CookieJar};
use chrono::{DateTime, Utc};
use http::{HeaderMap, header};
use std::{
    collections::{HashMap, hash_map::Values},
    sync::atomic::{AtomicU32, Ordering},
};

lazy_static::lazy_static! {
    /// Global context counter such that user session handles are unique
    /// even across different security sessions. That way, a handle from a
    /// previous session can never mistakenly be valid for a new session.
    static ref CONTEXT_COUNTER: AtomicU32 = AtomicU32::new(0);
}

/// Manages a security session, identified by the `SAP_SESSIONID_xxx` cookie.
///
/// All User Sessions, Cookies and the CSRF Token are bound to the security session.
/// While `stateless` request only requires the correct cookies to be present,
/// for `stateful` requests, the individual [`UserSession`] are maintained.
///
/// Once a security session is established, it is not needed to authenticate
/// with the system through an authorization header anymore as it can use
/// the security session cookies to validate our authorization.
///
/// See [HTTP Security Sessions](https://help.sap.com/docs/SAP_INTEGRATED_BUSINESS_PLANNING/685fbd2d5f8f4ca2aacfc35f1938d1c1/c7379ecf6a8f4c0bb09e88142124c77f.html?locale=en-US)
#[derive(Debug)]
pub(crate) struct SecuritySession {
    /// Timestamp of when this session was started
    start_time: DateTime<Utc>,

    /// Cookie Jar of this specific session.
    ///
    /// The `sap-contextid` cookie will not be included in this jar as it
    /// makes no sense for stateless sessions.
    cookies: CookieJar,

    /// CSRF Token required for most POST Endpoints, bound to the session.
    csrf_token: Option<String>,

    /// The contexts of this session, required for stateful communication.
    ///
    /// A stateful context must, for example, be held alive for the duration
    /// an object should remain locked. For short operations that require
    /// stateful sessions, it is recommended to create a seperate context
    /// and quickly discard it otherwise to avoid needlessly busy work processes.
    contexts: HashMap<UserSessionId, UserSession>,
}

impl SecuritySession {
    /// Creates a security session from the headers of a response.
    ///
    /// This assumes the presence of the required `set-cookie` headers.
    pub fn create_from_headers(headers: &HeaderMap, ctx: Option<UserSessionId>) -> Self {
        let mut jar = CookieJar::new();
        let mut contexts = HashMap::new();
        jar.set_from_multiple_headers(headers.get_all(header::SET_COOKIE));

        let csrf_token = headers
            .get(Cookie::CSRF_TOKEN)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        // The context id initially goes into the headers because its listed as a "set-cookie".
        // To allow multiple contexts to exist witin the same sesson, maintain them seperately.
        if let (Some(id), Some(cookie)) = (ctx, jar.take(Cookie::CONTEXT_ID)) {
            contexts.insert(id, UserSession::new(id, cookie));
        }

        Self {
            start_time: Utc::now(),
            cookies: jar,
            csrf_token: csrf_token,
            contexts: contexts,
        }
    }

    /// Updates the session data from the headers of a response.
    ///
    /// Modifications to cookies happen based to on the `set-cookie` headers,
    /// if a cookie is set to be expired, it is automatically removed from the jar.
    pub async fn update_from_headers(&mut self, headers: &HeaderMap, ctx: Option<UserSessionId>) {
        if let Some(csrf) = headers.get("x-csrf-token") {
            self.csrf_token = csrf.to_str().ok().map(|v| v.to_owned());
        }

        let cookie_headers = headers.get_all(header::SET_COOKIE);
        self.cookies.set_from_multiple_headers(cookie_headers);

        // The context id initially goes into the headers because its listed as a "set-cookie".
        // To allow multiple contexts to exist witin the same sesson, maintain them seperately.
        if let (Some(id), Some(cookie)) = (ctx, self.cookies.take(Cookie::CONTEXT_ID)) {
            if let Some(data) = self.contexts.get_mut(&id) {
                data.update(cookie)
            } else {
                self.contexts.insert(id, UserSession::new(id, cookie));
            }
        }
    }

    /// Gets the Session ID in the `SAP_SESSIONID_XXX` cookie if present.
    pub fn session_id(&self) -> Option<&str> {
        self.cookies.find(Cookie::SESSIONID).map(|v| v.value())
    }

    /// Whether the session has a CSRF Token for POST requests present.
    pub fn has_csrf_token(&self) -> bool {
        self.csrf_token.is_some()
    }

    /// The csrf token of the session for POST requests, if present.
    pub fn csrf_token(&self) -> Option<&String> {
        self.csrf_token.as_ref()
    }

    /// Bundles the statless cookies into a cookie header value to be used.
    ///
    /// Only cookies that match the destination are included.
    pub fn stateless_cookies(&self, destination: &str) -> String {
        self.cookies.to_header(destination)
    }

    /// Bundles the stateful cookies into a cookie header value to be used.
    ///
    /// In addition to the stateless cookies, the `sap-contextid` of the
    /// [`UserSession`] is added to the cookies.
    ///
    /// Only cookies that match the destination are included.
    pub fn stateful_cookies(&self, ctx: UserSessionId, destination: &str) -> String {
        let mut cookies = self.cookies.to_header(destination);
        if let Some(data) = self.contexts.get(&ctx) {
            cookies += &data.cookie().as_cookie_pair();
        }
        cookies
    }

    /// Gets the [`CookieJar`] of this security session.
    pub fn cookies(&self) -> &CookieJar {
        &self.cookies
    }

    /// Gets an iterator over the [`UserSession`] of this security session.
    pub fn user_sessions(&self) -> Values<UserSessionId, UserSession> {
        self.contexts.values()
    }

    /// Drops and returns a [`UserSession`] by it's handle.
    ///
    /// **Note:** This does not automatically drop the user session on the server.
    pub fn drop_user_session(&mut self, id: UserSessionId) -> Option<UserSession> {
        self.contexts.remove(&id)
    }
}

/// A unique identifier for a user session within a security session.
///
/// IDs are assigned incrementally, starting from 0, and are unique.
/// This identifier has no meaning for the server, its purely a means of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserSessionId(pub(crate) u32);

impl UserSessionId {
    pub fn next() -> Self {
        Self(CONTEXT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1)
    }
}

/// Represents a user session within a security session.
///
/// Hold a work process alive for its duration and thus retains a 'User Context'
/// which is the data belonging to the user session.
///
/// This is, for example, required to keep an ABAP Object locked across multiple calls
/// to the ADT backend. If the user context was not maintained, the object locks
/// would immediately be released and no persistent locking would take place.
///
/// See [User Sessions](https://help.sap.com/docs/ABAP_PLATFORM_BW4HANA/f146e75588924fa4987b6c8f1a7a8c7e/b7c55d0eaf5b4c6b91e4fbf7760c95e7.html?locale=en-US)
/// and [User Context](https://help.sap.com/docs/ABAP_PLATFORM_BW4HANA/f146e75588924fa4987b6c8f1a7a8c7e/c39c586b9f194454bee9ddb1e00a29ae.html?locale=en-US)
#[derive(Debug)]
pub(crate) struct UserSession {
    // ID of the context, serves as internal handle to the context.
    id: UserSessionId,

    // When was this context created? Not related to its first usage.
    created: DateTime<Utc>,

    // The cookie that represents this context in the request
    cookie: Cookie,
}

impl UserSession {
    /// Constructs a new user session with the given id and the context cookie.
    fn new(id: UserSessionId, cookie: Cookie) -> Self {
        Self {
            id,
            cookie,
            created: Utc::now(),
        }
    }

    /// The `sap-contextid` cookie that represents this user session.
    pub fn cookie(&self) -> &Cookie {
        &self.cookie
    }

    /// Updates the `sap-contextid` cookie with the new cookie value.
    fn update(&mut self, cookie: Cookie) {
        self.cookie = cookie;
    }
}
