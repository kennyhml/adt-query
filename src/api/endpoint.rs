use crate::api;
use crate::{
    ContextId, CookieJar, RequestBody, ResponseBody, Session, StatefulDispatch, StatelessDispatch,
    error::QueryError,
};
use async_trait::async_trait;
use http::request::Builder as RequestBuilder;
use http::{HeaderMap, Response};
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
    /// The type of request body of this endpoint, can be any serializable structure or unit ().
    type RequestBody: RequestBody;

    /// The type of response body of this endpoint, can be any deserializable structure or unit ().
    type ResponseBody: ResponseBody;

    /// The Kind of this endpoint, either [`Stateless`] or [`Stateful`] - marker type.
    type Kind: EndpointKind;

    /// The associated [`http::Method`] of this endpoint, e.g. `GET`, `POST`, `PUT`..
    const METHOD: http::Method;

    /// The relative URL for the query of this endpoint, outgoing from the system.
    ///
    /// Either a static URL, such as `/sap/bc/adt/core/discovery` or with path parameters:
    /// `/sap/bc/adt/programs/{z_some_program}/source`
    fn url(&self) -> Cow<'static, str>;

    /// The body to be included in the request, can be `None` if no body is desired.
    ///
    /// Otherwise, it can be any type that can later be deserialized into a body.
    fn body(&self) -> Option<&Self::RequestBody> {
        None
    }

    /// Extra headers to be included in the request, may be `None`.
    ///
    /// Common headers, such as session and context, are included independently.
    fn headers(&self) -> Option<&HeaderMap> {
        None
    }

    /// Content Type of the request, may be none if the body is also none.
    fn content_type(&self) -> Option<&'static str> {
        None
    }
}

/// Any Endpoint where `Kind = Stateless` implements the `StatelessQuery` trait
#[async_trait]
impl<E, T> api::StatelessQuery<T, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    T: StatelessDispatch,
{
    #[instrument(skip(self, client), level = Level::INFO)]
    async fn query(&self, client: &T) -> Result<Response<E::ResponseBody>, QueryError> {
        event!(
            Level::INFO,
            "{}: {} {}",
            client.info(),
            Self::METHOD,
            self.url()
        );

        let (request, cookie_guard) = build_request(self, client).await?;
        let body = build_body(&self.body())?;

        let response = client.dispatch(request, body).await?;
        update_cookies_from_response(client, response.headers(), cookie_guard).await;

        let (parts, body) = response.into_parts();

        if parts.status != 200 {
            return Err(QueryError::BadStatusCode {
                code: parts.status,
                message: body,
            });
        }
        Ok(Response::from_parts(parts, serde_xml_rs::from_str(&body)?))
    }
}

/// Any Endpoint where `Kind = Stateful` implements the `StatefulQuery` trait
#[async_trait]
impl<E, T> api::StatefulQuery<T, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateful> + Sync + Send,
    T: StatefulDispatch,
{
    async fn query(
        &self,
        client: &T,
        context: ContextId,
    ) -> Result<Response<E::ResponseBody>, QueryError> {
        event!(
            Level::INFO,
            "{}: {} {}",
            client.info(),
            Self::METHOD,
            self.url()
        );

        let (mut request, cookie_guard) = build_request(self, client).await?;
        api::inject_request_context(request.headers_mut().unwrap(), client, context).await?;

        let body = build_body(&self.body())?;

        let response = client.dispatch(request, body).await?;
        update_cookies_from_response(client, response.headers(), cookie_guard).await;

        let (parts, body) = response.into_parts();
        Ok(Response::from_parts(parts, serde_xml_rs::from_str(&body)?))
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
    endpoint: &E,
    session: &'a S,
) -> Result<(RequestBuilder, Option<MutexGuard<'a, CookieJar>>), QueryError>
where
    S: Session,
    E: Endpoint,
{
    let destination = session.destination();
    let uri = destination.server_url().join(&endpoint.url())?;
    let csrf = session.csrf_token().load_full().map(|v| v.as_ref().clone());

    if csrf.is_none() && E::METHOD != http::Method::GET {
        return Err(QueryError::MissingCsrfToken);
    }

    let mut req = http::request::Builder::new()
        .method(E::METHOD)
        .uri(uri.as_str())
        .version(http::Version::HTTP_11)
        .header("x-csrf-token", csrf.unwrap_or(String::from("fetch")));

    if let Some(content_type) = endpoint.content_type() {
        req = req.header("Content-Type", content_type);
    }

    // TODO: Is there a cleaner way to do this? Also consider performance. If the
    // headers are static, should really be copying them...
    if let Some(headers) = endpoint.headers() {
        for (k, v) in headers {
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

/// Helper method to build the request body. More accurately, this method
/// makes sure to deserialize the endpoint body if provided and convert
/// it to a byte stream for the dispatch method to accept.
fn build_body<T>(body: &Option<&T>) -> Result<Option<String>, QueryError>
where
    T: RequestBody,
{
    let config = serde_xml_rs::SerdeXml::new()
        .namespace("chkrun", "http://www.sap.com/adt/checkrun")
        .namespace("adtcore", "http://www.sap.com/adt/core");

    Ok(body
        .as_ref()
        .map(|body| config.to_string(body))
        .transpose()?
        .map(|s| s.to_string()))
}
