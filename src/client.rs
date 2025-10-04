use crate::RequestDispatch;
use crate::error::{DispatchError, OperationError};
use crate::session::{SecuritySession, UserSessionId};
use crate::{ConnectionParameters, auth::Credentials};

use async_trait::async_trait;
use derive_builder::Builder;
use http::request::Builder as RequestBuilder;
use http::{Method, Response, header};
use tokio::sync::{Mutex as AsyncMutex, MutexGuard};
use url::Url;

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct Client<T>
where
    T: RequestDispatch,
{
    /// Request dispatch implementation, may be user defined and use either
    /// HTTP or RFC to handle the final communication with the backend system.
    dispatcher: T,

    #[builder(setter(name = "connection_params", strip_option))]
    params: ConnectionParameters,

    #[builder(setter(skip))]
    session: AsyncMutex<Option<SecuritySession>>,

    #[builder(setter(skip))]
    session_init_guard: AsyncMutex<()>,

    credentials: Credentials,

    /// Number of requests this client has dispatched
    #[builder(setter(skip), default = 0)]
    dispatch_count: i32,
}

impl<T> Client<T>
where
    T: RequestDispatch,
{
    /// Terminates the current security session, if one exists.
    ///
    /// Upon successful termination:
    /// - All associated user sessions (contexts) are cleaned up by the server.
    /// - Any resources or objects locked by the user sessions (contexts) are released.
    ///
    /// ## Returns
    /// Whether a session was active and subsequently destroyed
    ///
    /// ## Errors
    /// [`DispatchError`] if the request to destroy the session failed.
    pub async fn destroy_session(&self) -> Result<bool, OperationError> {
        if self.session.lock().await.is_none() {
            return Ok(false);
        }

        let request = RequestBuilder::new()
            .uri(
                self.params
                    .url()
                    .join("sap/public/bc/icf/logoff")
                    .unwrap()
                    .to_string(),
            )
            .method(Method::POST);

        self.dispatch_stateless(request, String::new()).await?;
        Ok(true)
    }

    pub async fn dispatch_stateless(
        &self,
        request: RequestBuilder,
        body: String,
    ) -> Result<Response<String>, DispatchError> {
        let _guard = self.login_lock().await;

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
        ctx: UserSessionId,
    ) -> Result<Response<String>, DispatchError> {
        let _guard = self.login_lock().await;

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
            let dst = request.uri_ref().map(|v| v.to_string()).unwrap_or_default();
            request
                .header(header::COOKIE, session.stateless_cookies(&dst))
                .header("x-csrf-token", session.csrf_token().map_or("fetch", |v| &v))
        } else {
            request
                .header("x-csrf-token", "fetch")
                .header(header::AUTHORIZATION, self.credentials.basic_auth())
        }
    }

    async fn add_stateful_headers(
        &self,
        request: RequestBuilder,
        ctx: UserSessionId,
    ) -> RequestBuilder {
        let request = request.header("x-sap-adt-sessiontype", "stateful");
        if let Some(session) = self.session.lock().await.as_ref() {
            let dst = request.uri_ref().map(|v| v.to_string()).unwrap_or_default();
            request
                .header(header::COOKIE, session.stateful_cookies(ctx, &dst))
                .header("x-csrf-token", session.csrf_token().map_or("fetch", |v| &v))
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
                .map_or(true, |s| !s.has_csrf_token())
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

    async fn update_from_response(&self, response: &Response<String>, ctx: Option<UserSessionId>) {
        // Avoid locking if there are no headers to update anyway.
        if !response.headers().contains_key(header::SET_COOKIE) {
            return;
        }

        let mut session_guard = self.session.lock().await;
        if let Some(session) = session_guard.as_mut() {
            session.update_from_headers(response.headers(), ctx).await;
            // All cookies were destroyed, the session was invalidated
            if session.cookies().is_empty() {
                *session_guard = None;
            }
        } else {
            let session = SecuritySession::create_from_headers(response.headers(), ctx);
            *session_guard = Some(session);
        }
    }

    pub fn destination(&self) -> &Url {
        &self.params.url()
    }

    pub async fn session_id(&self) -> Option<String> {
        self.session
            .lock()
            .await
            .as_ref()
            .and_then(|v| v.session_id().map(|v| v.to_string()))
    }

    pub fn create_user_session(&self) -> UserSessionId {
        UserSessionId::next()
    }

    pub async fn destroy_user_session(&self, id: UserSessionId) -> Result<bool, DispatchError> {
        let mut session = self.session.lock().await;

        let session = match session.as_mut() {
            Some(s) => s,
            None => return Ok(false),
        };
        let ctx = match session.drop_user_session(id) {
            Some(c) => c,
            None => return Ok(false),
        };

        let mut cookies = session.stateless_cookies("");
        cookies += &ctx.cookie().as_cookie_pair();

        let req = RequestBuilder::new()
            .uri(self.params.url().join("sap/bc/adt")?.to_string())
            .method(Method::POST)
            .header("x-sap-adt-sessiontype", "stateless")
            .header(header::COOKIE, cookies);
        self.dispatcher.dispatch_request(req, String::new()).await?;
        Ok(true)
    }

    async fn login_lock(&self) -> Option<MutexGuard<'_, ()>> {
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

    use crate::HttpConnectionBuilder;

    use super::*;

    fn test_client() -> Client<reqwest::Client> {
        let params = HttpConnectionBuilder::default()
            .hostname(Url::from_str("http://localhost:50000").unwrap())
            .client("001")
            .language("en")
            .build()
            .unwrap();

        ClientBuilder::default()
            .connection_params(ConnectionParameters::Http(params))
            .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
            .dispatcher(reqwest::Client::new())
            .build()
            .unwrap()
    }

    #[test]
    fn distinct_user_sessions_get_created() {
        let client = test_client();

        let first_contex = client.create_user_session();
        let second_context = client.create_user_session();

        assert_ne!(
            first_contex, second_context,
            "Context identifiers are not unique."
        );
    }

    #[test]
    fn user_session_creation_is_thread_safe() {
        let client = Arc::new(Mutex::new(test_client()));
        let contexts = Arc::new(Mutex::new(vec![]));
        let mut handles = vec![];

        for _ in 0..10 {
            let client = Arc::clone(&client);
            let contexts = Arc::clone(&contexts);
            let handle = thread::spawn(move || {
                let context = client.lock().unwrap().create_user_session();
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
