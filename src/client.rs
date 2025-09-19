use crate::error::DispatchError;
use crate::{Context, ContextId, CookieJar, System, auth::Credentials};
use crate::{Cookie, RequestDispatch};

use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::request::Builder as RequestBuilder;
use http::{HeaderMap, Response, header};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{Mutex as AsyncMutex, MutexGuard};

lazy_static::lazy_static! {
    /// Global context counter such that context handles are unique
    /// even across different sessions. That way, a handle from a
    /// previous session can never mistakenly be valid for a new session.
    static ref CONTEXT_COUNTER: AtomicU32 = AtomicU32::new(0);
}

/// Represents a user session on the SAP System. The session is determined
/// by the `SAP_SESSIONID_xxx` cookie. Stateful and Stateless requests
/// can both be used in the context of that same session, but the headers
/// must be managed accordingly.
#[derive(Debug)]
pub struct UserSession {
    /// Timestamp of when this session started on the backend
    start_time: DateTime<Utc>,

    /// Cookie Jar of this specific session.
    ///
    /// The `sap-contextid` cookie will not be included in this jar as it
    /// makes no sense for stateless sessions.
    cookies: CookieJar,

    /// CSRF Token required for most POST Endpoints, bound to the session.
    csrf_token: ArcSwapOption<String>,

    /// The contexts of this session, required for stateful communication.
    ///
    /// A stateful context must, for example, be held alive for the duration
    /// an object should remain locked. For short operations that require
    /// stateful sessions, it is recommended to create a seperate context
    /// and quickly discard it otherwise to avoid needlessly busy work processes.
    contexts: AsyncMutex<HashMap<ContextId, Context>>,
}

impl UserSession {
    fn create_from_headers(headers: &HeaderMap, ctx: Option<ContextId>) -> Self {
        let mut jar = CookieJar::new();
        let mut contexts = HashMap::new();
        jar.set_from_multiple_headers(headers.get_all(Cookie::SET_COOKIE));

        let csrf = headers
            .get(Cookie::CSRF_TOKEN)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        // The contextid initially goes into the headers because its listed as a "set-cookie".
        // To allow multiple contexts to exist witin the same sesson, they must be held seperately.
        if let (Some(id), Some(cookie)) = (ctx, jar.take(Cookie::CONTEXT_ID)) {
            contexts.insert(id, Context::new(id, cookie));
        }

        Self {
            start_time: Utc::now(),
            cookies: jar,
            csrf_token: ArcSwapOption::from_pointee(csrf),
            contexts: AsyncMutex::new(contexts),
        }
    }

    async fn update_from_headers(&mut self, headers: &HeaderMap, ctx: Option<ContextId>) {
        if let Some(csrf) = headers.get("x-csrf-token") {
            self.csrf_token.swap(Some(Arc::new(
                csrf.to_str().ok().map(|v| v.to_owned()).unwrap_or_default(),
            )));
        }
        self.cookies
            .set_from_multiple_headers(headers.get_all(Cookie::SET_COOKIE));

        // The contextid initially goes into the headers because its listed as a "set-cookie".
        // To allow multiple contexts to exist witin the same sesson, they must be held seperately.
        if let (Some(id), Some(cookie)) = (ctx, self.cookies.take(Cookie::CONTEXT_ID)) {
            if let Some(data) = self.contexts.lock().await.get_mut(&id) {
                data.update(cookie)
            }
        }
    }

    fn stateless_cookies(&self) -> String {
        self.cookies.to_header("")
    }

    async fn stateful_cookies(&self, ctx: ContextId) -> String {
        let mut cookies = self.cookies.to_header("");
        if let Some(data) = self.contexts.lock().await.get(&ctx) {
            cookies += &data.cookie().as_cookie_pair();
        }
        cookies
    }

    fn csrf_header(&self) -> String {
        self.csrf_token
            .load_full()
            .map(|v| v.as_ref().clone())
            .unwrap_or_default()
    }

    async fn insert_context(&self, id: ContextId, cookie: Cookie) {
        let mut contexts = self.contexts.lock().await;
        contexts.insert(id, Context::new(id, cookie));
    }

    async fn contexts(&self) -> &AsyncMutex<HashMap<ContextId, Context>> {
        &self.contexts
    }

    async fn drop_context(&self, id: ContextId) -> Option<Context> {
        self.contexts.lock().await.remove(&id)
    }
}

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct Client<T>
where
    T: RequestDispatch,
{
    /// Request dispatch implementation, may be user defined and use either
    /// HTTP or RFC to handle the final communication with the backend system.
    dispatcher: T,

    /// The SAP System this client is connecting / connected with.
    system: System,

    /// The client number that we are connecting / connected to the SAP System with.
    client: i32,

    #[builder(setter(skip))]
    session: AsyncMutex<Option<UserSession>>,

    #[builder(setter(skip))]
    session_init_guard: AsyncMutex<()>,

    #[builder(setter(into))]
    language: String,

    credentials: Credentials,

    /// Number of requests this client has dispatched
    #[builder(setter(skip), default = 0)]
    dispatch_count: i32,
}

impl<T> Client<T>
where
    T: RequestDispatch,
{
    pub async fn dispatch_stateless(
        &self,
        request: RequestBuilder,
        body: String,
    ) -> Result<Response<String>, DispatchError> {
        let _guard = self.session_init_guard().await;

        let mut request = request.header("x-sap-adt-sessiontype", "stateless");
        if let Some(session) = self.session.lock().await.as_ref() {
            request = request
                .header(header::COOKIE, session.stateless_cookies())
                .header("x-csrf-token", session.csrf_header())
        } else {
            request = request
                .header("x-csrf-token", "fetch")
                .header(header::AUTHORIZATION, self.credentials.basic_auth())
        }

        let res = self.dispatcher.dispatch_request(request, body).await?;
        self.update_session_from_response(&res, None).await;
        Ok(res)
    }

    pub async fn dispatch_stateful(
        &self,
        request: RequestBuilder,
        body: String,
        ctx: ContextId,
    ) -> Result<Response<String>, DispatchError> {
        let _guard = self.session_init_guard().await;

        let mut request = request.header("x-sap-adt-sessiontype", "stateful");
        if let Some(session) = self.session.lock().await.as_ref() {
            request = request
                .header(header::COOKIE, session.stateful_cookies(ctx).await)
                .header("x-csrf-token", session.csrf_header())
        } else {
            request = request
                .header("x-csrf-token", "fetch")
                .header(header::AUTHORIZATION, self.credentials.basic_auth())
        }

        let res = self.dispatcher.dispatch_request(request, body).await?;
        self.update_session_from_response(&res, Some(ctx)).await;
        Ok(res)
    }

    async fn update_session_from_response(
        &self,
        response: &Response<String>,
        ctx: Option<ContextId>,
    ) {
        // Avoid locking if there are no headers to update anyway.
        if !response.headers().contains_key(Cookie::SET_COOKIE) {
            return;
        }

        let mut session_guard = self.session.lock().await;
        if let Some(session) = session_guard.as_mut() {
            session.update_from_headers(response.headers(), ctx).await;
        } else {
            let session = UserSession::create_from_headers(response.headers(), ctx);
            *session_guard = Some(session);
        }
    }

    pub fn destination(&self) -> &System {
        &self.system
    }

    pub fn client(&self) -> i32 {
        self.client
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn reserve_context(&self) -> ContextId {
        let new_value = CONTEXT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        ContextId(new_value)
    }

    pub async fn drop_context(&self, id: ContextId) -> Result<bool, DispatchError> {
        if let Some(session) = self.session.lock().await.as_mut() {
            if let Some(_ctx) = session.drop_context(id).await {
                //  Make a HTTP request to actually drop the context
                return Ok(true);
            }
        }
        Ok(false)
    }

    async fn session_init_guard(&self) -> Option<MutexGuard<'_, ()>> {
        if self.session.lock().await.is_some() {
            return None;
        }
        let guard = self.session_init_guard.lock().await;
        if self.session.lock().await.is_none() {
            Some(guard)
        } else {
            // Session was created while waiting for the lock
            drop(guard);
            None
        }
    }

    fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl RequestDispatch for reqwest::Client {
    async fn dispatch_request(
        &self,
        request: RequestBuilder,
        body: String,
    ) -> Result<Response<String>, DispatchError> {
        let request = request.body(body)?;
        let (parts, body) = request.into_parts();

        let response = self
            .request(parts.method, parts.uri.to_string())
            .body(body)
            .headers(parts.headers)
            .send()
            .await?;

        let mut mapped = Response::builder().status(response.status());
        if let Some(headers) = mapped.headers_mut() {
            *headers = response.headers().clone();
        }
        Ok(mapped.body(response.text().await?)?)
    }
}

#[cfg(test)]
pub mod tests {
    // use std::collections::HashSet;
    // use std::str::FromStr as _;

    // use std::sync::{Arc, Mutex};
    // use std::thread;
    // use url::Url;

    // use crate::SystemBuilder;

    // use super::*;

    // fn test_client() -> Client {
    //     let system = SystemBuilder::default()
    //         .name("A4H")
    //         .server_url(Url::from_str("http://localhost:50000").unwrap())
    //         .build()
    //         .unwrap();

    //     ClientBuilder::default()
    //         .system(system)
    //         .language("en")
    //         .client(001)
    //         .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
    //         .build()
    //         .unwrap()
    // }

    // #[test]
    // fn distinct_contexts_get_created() {
    //     let client = test_client();

    //     let first_contex = client.create_context();
    //     let second_context = client.create_context();

    //     assert_ne!(
    //         first_contex, second_context,
    //         "Context identifiers are not unique."
    //     );
    // }

    // #[tokio::test]
    // async fn context_gets_inserted() {
    //     let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

    //     let client = test_client();

    //     let context_id = client.create_context();
    //     client.insert_context(context_id, Cookie::parse(cookie).unwrap());

    //     let ctx = client.context(context_id);
    //     assert!(ctx.is_some(), "Context was not inserted");
    //     assert_eq!(
    //         ctx.unwrap().lock().await.cookie().value(),
    //         "SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT"
    //     )
    // }

    // #[tokio::test]
    // async fn context_gets_dropped() {
    //     let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

    //     let client = test_client();

    //     let context_id = client.create_context();
    //     client.insert_context(context_id, Cookie::parse(cookie).unwrap());
    //     client.drop_context(context_id);

    //     let ctx = client.context(context_id);

    //     assert!(ctx.is_none(), "Context was not dropped");
    // }

    // #[test]
    // fn context_reservation_is_thread_safe() {
    //     let client = Arc::new(Mutex::new(test_client()));
    //     let contexts = Arc::new(Mutex::new(vec![]));
    //     let mut handles = vec![];

    //     for _ in 0..10 {
    //         let client = Arc::clone(&client);
    //         let contexts = Arc::clone(&contexts);
    //         let handle = thread::spawn(move || {
    //             let context = client.lock().unwrap().create_context();
    //             contexts.lock().unwrap().push(context);
    //         });
    //         handles.push(handle);
    //     }

    //     // Wait for all threads to complete
    //     for handle in handles {
    //         handle.join().unwrap();
    //     }

    //     let set: HashSet<_> = contexts.lock().unwrap().drain(..).collect();
    //     assert_eq!(set.len(), 10, "Not all context ids are unique.");
    // }
}
