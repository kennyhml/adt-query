use crate::error::{BadRequest, ResponseError, SerializeError};
use crate::query::{StatefulQuery, StatelessQuery, inject_request_context};
use crate::{
    ContextId, CookieJar, Session, StatefulDispatch, StatelessDispatch, error::QueryError,
};
use crate::{Contextualize, QueryParameters};
use async_trait::async_trait;
use http::HeaderMap;
use http::request::Builder as RequestBuilder;
use std::borrow::Cow;
use std::sync::Arc;
use tokio::sync::MutexGuard;
use tracing::{Level, event, instrument};

pub trait EndpointKind {}

pub struct Stateless {}
pub struct Stateful {}

impl EndpointKind for Stateful {}
impl EndpointKind for Stateless {}

/// An endpoint on the SAP System that can be called.
///
/// The implementing structure controls the request url, parameters and headers. Endpoints can
/// either be `Stateless` or `Stateful`. Check [`crate::core::Context`] for more information
/// on stateful endpoints.
///
/// In that sense, instances of endpoints can be seen as the prelude of a request to that endpoint.
///
/// The [`api::StatelessQuery`] or [`api::StatefulQuery`] traits are automatically implemented
/// for types that implement [`Endpoint`] depending on the associated `Kind` Type.
pub trait Endpoint {
    /// The type of response body of this endpoint, can be any deserializable structure or unit ().
    type Response: TryFrom<http::Response<String>, Error = ResponseError>;

    /// The Kind of this endpoint, either [`Stateless`] or [`Stateful`] - marker type.
    type Kind: EndpointKind;

    /// The associated [`http::Method`] of this endpoint, e.g. `GET`, `POST`, `PUT`..
    const METHOD: http::Method;

    /// The relative URL for the query of this endpoint, outgoing from the system host.
    ///
    /// **Warning:** Use the [`parameters()`](method@parameters) method for query parameters.
    fn url(&self) -> Cow<'static, str>;

    /// The body to be included in the request, can be `None` if no body is desired. For more
    /// flexibility, this can be any string that is later passed into the request body.
    ///
    /// Remark: The request will inevitably have to clone the data anyway, so moving is fine.
    fn body(&self) -> Option<Result<String, SerializeError>> {
        None
    }

    /// The query parameters to be added to the query.
    fn parameters(&self) -> QueryParameters {
        QueryParameters::default()
    }

    /// Extra headers to be included in the request, may be `None`.
    ///
    /// Common headers, such as session and context, are included independently.
    fn headers(&self) -> Option<HeaderMap> {
        None
    }
}

/// Any Endpoint where `Kind = Stateless` implements the `StatelessQuery` trait
#[async_trait]
impl<E, T> StatelessQuery<T, E::Response> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    T: StatelessDispatch,
{
    #[instrument(skip(self, client), level = Level::INFO)]
    async fn query(&self, client: &T) -> Result<E::Response, QueryError> {
        event!(
            Level::INFO,
            "{}: {} {}",
            client.info(),
            Self::METHOD,
            self.url()
        );

        let (mut request, cookie_guard) = build_request(self, client).await?;

        // We might need to send a GET request first to obtain a CSRF token.
        // Luckily, even if the request is rejected semantically, we are still
        // provided with the cookies and csrf token, so concurrency is not an issue.
        if E::METHOD != http::Method::GET && client.csrf_token().load().is_none() {
            event!(Level::DEBUG, "Must first GET to obtain a CSRF-Token.");
            request = request.method(http::Method::GET);
            let response = client.dispatch(request.body(Vec::new())?).await?;
            update_cookies_from_response(client, response.headers(), cookie_guard).await;

            // Try POST again, kind of a shitty hack for now, cant clone the request builder.. :c
            return self.query(client).await;
        }

        let body = self
            .body()
            .transpose()
            .map_err(BadRequest::SerializeError)?
            .unwrap_or_default()
            .into_bytes();
        let response = client.dispatch(request.body(body)?).await?;
        if response.status() == 401 {
            return Err(QueryError::Unauthorized);
        }

        update_cookies_from_response(client, response.headers(), cookie_guard).await;
        Ok(E::Response::try_from(response)?)
    }
}

/// Any Endpoint where `Kind = Stateful` implements the `StatefulQuery` trait
#[async_trait]
impl<'a, E, T> StatefulQuery<T, E::Response> for E
where
    E: Endpoint<Kind = Stateful> + Sync + Send,
    T: StatefulDispatch,
{
    async fn query(&self, client: &T, context: ContextId) -> Result<E::Response, QueryError> {
        event!(
            Level::INFO,
            "{}: {} {}",
            client.info(),
            Self::METHOD,
            self.url()
        );

        let (mut request, cookie_guard) = build_request(self, client).await?;
        // We might need to send a GET request first to obtain a CSRF token.
        // Luckily, even if the request is rejected semantically, we are still
        // provided with the cookies and csrf token, so concurrency is not an issue.
        if E::METHOD != http::Method::GET && client.csrf_token().load().is_none() {
            event!(Level::DEBUG, "Must first GET to obtain a CSRF-Token.");
            request = request.method(http::Method::GET);
            let response = client.dispatch(request.body(Vec::new())?).await?;
            update_session_from_response(client, response.headers(), cookie_guard, context).await;

            // Try POST again, kind of a shitty hack for now, cant clone the request builder.. :c
            return self.query(client, context).await;
        }

        inject_request_context(request.headers_mut().unwrap(), client, context).await?;

        let body = self
            .body()
            .transpose()
            .map_err(BadRequest::SerializeError)?
            .unwrap_or_default()
            .into_bytes();
        let response = client.dispatch(request.body(body)?).await?;
        update_session_from_response(client, response.headers(), cookie_guard, context).await;
        Ok(E::Response::try_from(response)?)
    }
}

/// Helper method to build the fundamental request from an endpoint.
///
/// As stateless and stateful queries use the same foundational properties,
/// both may use this method to take care of the basic chores.
///
/// In the case of the first logon to the system, i.e. no prior session id exists,
/// the cookie jar mutex guard is passed back to the caller to keep it alive. This
/// must be done to ensure that no concurrent request occurs with an empty jar which
/// would then iniate a second session creation.
async fn build_request<'a, S, E>(
    endpoint: &'a E,
    session: &'a S,
) -> Result<(RequestBuilder, Option<MutexGuard<'a, CookieJar>>), QueryError>
where
    S: Session,
    E: Endpoint,
{
    let destination = session.destination();
    let mut uri = destination.server_url().join(&endpoint.url())?;
    endpoint.parameters().add_to_url(&mut uri);

    let csrf = session.csrf_token().load_full().map(|v| v.as_ref().clone());

    let mut req = http::request::Builder::new()
        .method(E::METHOD)
        .uri(uri.as_str())
        .version(http::Version::HTTP_11)
        .header("x-csrf-token", csrf.unwrap_or(String::from("fetch")));

    if let Some(headers) = endpoint.headers() {
        for (k, v) in headers.iter() {
            req = req.header(k, v);
        }
    }

    let cookies = session.cookies().lock().await;
    // If there is no session cookie yet, we must hold the lock to the cookies
    // until the request has completed. Otherwise, another (concurrent) request
    // could end up establishing another session and racing occurs.
    let cookie_guard: Option<MutexGuard<'a, CookieJar>> = if cookies.is_empty() {
        req = req.header("Authorization", session.credentials().basic_auth());
        Some(cookies)
    } else {
        req = req.header("Cookie", cookies.to_header(&uri)?);
        None
    };
    Ok((req, cookie_guard))
}

/// Updates the session cookies from the `set-cookie` headers in the response.
///
/// After this step is done, this is also where the cookie jar mutex guard will
/// be dropped in any case and is available for concurrent access going forward.
async fn update_cookies_from_response<'a, S>(
    session: &'a S,
    response_headers: &HeaderMap,
    existing_guard: Option<MutexGuard<'a, CookieJar>>,
) where
    S: Session,
{
    if let Some(csrf_token) = response_headers.get("x-csrf-token") {
        session
            .csrf_token()
            .store(Some(Arc::new(csrf_token.to_str().unwrap().to_owned())));
    }

    // No cookies to update, avoid locking the jar where possible.
    if !response_headers.contains_key("set-cookie") {
        return;
    }
    let mut cookies = match existing_guard {
        Some(lock) => lock,
        None => session.cookies().lock().await,
    };

    cookies.set_from_multiple_headers(response_headers.get_all("set-cookie"));
}

/// Updates the session cookies from the `set-cookie` headers in the response.
///
/// After this step is done, this is also where the cookie jar mutex guard will
/// be dropped in any case and is available for concurrent access going forward.
async fn update_session_from_response<'a, S>(
    session: &'a S,
    response_headers: &HeaderMap,
    existing_guard: Option<MutexGuard<'a, CookieJar>>,
    context: ContextId,
) where
    S: Session + Contextualize,
{
    if let Some(csrf_token) = response_headers.get("x-csrf-token") {
        session
            .csrf_token()
            .store(Some(Arc::new(csrf_token.to_str().unwrap().to_owned())));
    }

    // No cookies to update, avoid locking the jar where possible.
    if !response_headers.contains_key("set-cookie") {
        return;
    }
    let mut cookies = match existing_guard {
        Some(lock) => lock,
        None => session.cookies().lock().await,
    };

    cookies.set_from_multiple_headers(response_headers.get_all("set-cookie"));
    if let Some(session_cookie) = cookies.take("sap-contextid") {
        match session.context(context) {
            Some(context) => context.lock().await.update(session_cookie),
            None => session.insert_context(context, session_cookie),
        }
    }
}

#[cfg(test)]

mod tests {

    use std::str::FromStr as _;

    use url::Url;

    use crate::{Client, ClientBuilder, SystemBuilder, auth::Credentials, response::Success};

    use super::*;

    struct SamplePostEndpoint {}

    impl Endpoint for SamplePostEndpoint {
        type Response = Success<()>;
        type Kind = Stateless;

        const METHOD: http::Method = http::Method::POST;

        fn url(&self) -> Cow<'static, str> {
            "sap/bc/some/url/that/doesnt/exist".into()
        }
    }

    fn test_client() -> Client {
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
            .build()
            .unwrap()
    }
}
