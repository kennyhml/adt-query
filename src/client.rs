use crate::error::DispatchError;
use crate::{Context, ContextId, CookieJar, System, auth::Credentials};
use crate::{Cookie, RequestDispatch};

use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::request::Builder as RequestBuilder;
use http::{HeaderMap, Method, Response, header};
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

type SessionGuardOpt<'a> = Option<MutexGuard<'a, Option<UserSession>>>;

/// Represents a user session on the SAP System. The session is determined
/// by the `SAP_SESSIONID_xxx` cookie. Stateful and Stateless requests
/// can both be used in the context of that same session, but the headers
/// must be managed accordingly.
#[derive(Debug)]
struct UserSession {
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
            let mut contexts = self.contexts.lock().await;
            if let Some(data) = contexts.get_mut(&id) {
                data.update(cookie)
            } else {
                contexts.insert(id, Context::new(id, cookie));
            }
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        self.cookies.find(Cookie::SESSIONID).map(|v| v.value())
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

    fn csrf_header_set(&self) -> bool {
        self.csrf_token.load_full().is_some()
    }

    fn csrf_header(&self) -> String {
        self.csrf_token
            .load_full()
            .map(|v| v.as_ref().clone())
            .unwrap_or(String::from("fetch"))
    }

    async fn insert_context(&self, id: ContextId, cookie: Cookie) {
        let mut contexts = self.contexts.lock().await;
        contexts.insert(id, Context::new(id, cookie));
    }

    pub async fn cookies(&self) -> &CookieJar {
        &self.cookies
    }

    pub async fn contexts(&self) -> &AsyncMutex<HashMap<ContextId, Context>> {
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

        // Prefetch csrf token by post request.
        if self.csrf_prefetch_required(&request).await {
            self.prefetch_csrf_token(&request).await?;
        }

        let request = self.add_stateless_headers(request).await;
        let res = self.dispatcher.dispatch_request(request, body).await?;
        self.update_from_response(&res, None).await;
        Ok(res)
    }

    pub async fn dispatch_stateful(
        &self,
        request: RequestBuilder,
        body: String,
        ctx: ContextId,
    ) -> Result<Response<String>, DispatchError> {
        let _guard = self.session_init_guard().await;

        // Prefetch csrf token by post request.
        if self.csrf_prefetch_required(&request).await {
            self.prefetch_csrf_token(&request).await?;
        }

        let request = self.add_stateful_headers(request, ctx).await;
        let res = self.dispatcher.dispatch_request(request, body).await?;
        self.update_from_response(&res, Some(ctx)).await;
        Ok(res)
    }

    async fn add_stateless_headers(&self, request: RequestBuilder) -> RequestBuilder {
        let request = request.header("x-sap-adt-sessiontype", "stateless");
        if let Some(session) = self.session.lock().await.as_ref() {
            request
                .header(header::COOKIE, session.stateless_cookies())
                .header("x-csrf-token", session.csrf_header())
        } else {
            request
                .header("x-csrf-token", "fetch")
                .header(header::AUTHORIZATION, self.credentials.basic_auth())
        }
    }

    async fn add_stateful_headers(
        &self,
        request: RequestBuilder,
        ctx: ContextId,
    ) -> RequestBuilder {
        let request = request.header("x-sap-adt-sessiontype", "stateful");
        if let Some(session) = self.session.lock().await.as_ref() {
            request
                .header(header::COOKIE, session.stateful_cookies(ctx).await)
                .header("x-csrf-token", session.csrf_header())
        } else {
            request
                .header("x-csrf-token", "fetch")
                .header(header::AUTHORIZATION, self.credentials.basic_auth())
        }
    }

    async fn csrf_prefetch_required(&self, request: &RequestBuilder) -> bool {
        request.method_ref().unwrap() == Method::POST
            && self
                .session
                .lock()
                .await
                .as_ref()
                .map_or(true, |s| !s.csrf_header_set())
    }

    async fn prefetch_csrf_token(&self, request: &RequestBuilder) -> Result<(), DispatchError> {
        let mut csrf_request = clone_as_csrf_request(&request);

        // Always use stateless for a csrf prefetch request!
        csrf_request = self.add_stateless_headers(csrf_request).await;

        let body = String::new();

        let res = self.dispatcher.dispatch_request(csrf_request, body).await?;
        self.update_from_response(&res, None).await;
        Ok(())
    }

    async fn update_from_response(&self, response: &Response<String>, ctx: Option<ContextId>) {
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

    pub async fn session_id(&self) -> Option<String> {
        self.session
            .lock()
            .await
            .as_ref()
            .and_then(|v| v.session_id().map(|v| v.to_string()))
    }

    pub fn create_context(&self) -> ContextId {
        let new_value = CONTEXT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
        ContextId(new_value)
    }

    pub async fn drop_context(&self, id: ContextId) -> Result<bool, DispatchError> {
        if let Some(session) = self.session.lock().await.as_mut() {
            if let Some(ctx) = session.drop_context(id).await {
                let mut request = RequestBuilder::new()
                    .uri(
                        self.system
                            .server_url()
                            .join("/sap/bc/adt")
                            .unwrap()
                            .to_string(),
                    )
                    .method(Method::POST)
                    .header("x-sap-adt-sessiontype", "stateless");
                let mut cookies = session.stateless_cookies();
                cookies += &ctx.cookie().as_cookie_pair();
                request = request.header("cookie", cookies);
                self.dispatcher
                    .dispatch_request(request, String::new())
                    .await?;
                // No need to update the session cookies
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
        println!("{:?}", request);
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

fn is_missing_csrf_token(request: &RequestBuilder) -> bool {
    if request.method_ref().unwrap() != Method::POST {
        return false;
    }
    request.headers_ref().map_or(true, |h| {
        h.get("x-csrf-token").map_or(true, |v| v == "fetch")
    })
}

fn clone_as_csrf_request(request: &RequestBuilder) -> RequestBuilder {
    let mut req = RequestBuilder::new()
        .method(Method::GET)
        .uri(request.uri_ref().clone().unwrap());

    if let Some(map) = request.headers_ref() {
        for (k, v) in map.iter() {
            req = req.header(k, v)
        }
    }
    req
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;
    use std::str::FromStr as _;

    use std::sync::{Arc, Mutex};
    use std::thread;
    use url::Url;

    use crate::SystemBuilder;

    use super::*;

    fn test_client() -> Client<reqwest::Client> {
        let system = SystemBuilder::default()
            .name("A4H")
            .server_url(Url::from_str("http://localhost:50000").unwrap())
            .build()
            .unwrap();

        ClientBuilder::default()
            .system(system)
            .language("en")
            .client(001)
            .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
            .dispatcher(reqwest::Client::new())
            .build()
            .unwrap()
    }

    #[test]
    fn distinct_contexts_get_created() {
        let client = test_client();

        let first_contex = client.create_context();
        let second_context = client.create_context();

        assert_ne!(
            first_contex, second_context,
            "Context identifiers are not unique."
        );
    }

    #[test]
    fn context_reservation_is_thread_safe() {
        let client = Arc::new(Mutex::new(test_client()));
        let contexts = Arc::new(Mutex::new(vec![]));
        let mut handles = vec![];

        for _ in 0..10 {
            let client = Arc::clone(&client);
            let contexts = Arc::clone(&contexts);
            let handle = thread::spawn(move || {
                let context = client.lock().unwrap().create_context();
                contexts.lock().unwrap().push(context);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let set: HashSet<_> = contexts.lock().unwrap().drain(..).collect();
        assert_eq!(set.len(), 10, "Not all context ids are unique.");
    }
}
