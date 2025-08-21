use crate::endpoint;
use crate::{
    ContextId, RequestBody, ResponseBody, Session, StatefulDispatch, StatelessDispatch,
    common::CookieJar, error::QueryError,
};
use async_trait::async_trait;
use http::request::Builder as RequestBuilder;
use http::{HeaderMap, Response};
use std::borrow::Cow;
use tokio::sync::MutexGuard;
use tracing::{Level, event, instrument};

pub trait EndpointKind {}

pub struct Stateless {}
pub struct Stateful {}

impl EndpointKind for Stateful {}
impl EndpointKind for Stateless {}

pub trait Endpoint {
    type RequestBody: RequestBody;
    type ResponseBody: ResponseBody;
    type Kind: EndpointKind;

    const METHOD: http::Method;

    fn url(&self) -> Cow<'static, str>;

    fn body(&self) -> Option<Self::RequestBody> {
        None
    }

    fn headers(&self) -> Option<&HeaderMap> {
        None
    }
}

#[async_trait]
pub trait StatelessQuery<T, R> {
    async fn query(&self, client: &T) -> Result<Response<R>, QueryError>;
}

#[async_trait]
pub trait StatefulQuery<T, R> {
    async fn query(&self, client: &T, context: ContextId) -> Result<Response<R>, QueryError>;
}

#[async_trait]
impl<E, T> StatelessQuery<T, E::ResponseBody> for E
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
        update_cookies_from_response(client, &response, cookie_guard).await;

        let (parts, body) = response.into_parts();
        Ok(Response::from_parts(
            parts,
            serde_xml_rs::from_str(std::str::from_utf8(&body).unwrap())?,
        ))
    }
}

#[async_trait]
impl<E, T> StatefulQuery<T, E::ResponseBody> for E
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
        println!("{:?}", context);

        let (request, cookie_guard) = build_request(self, client).await?;
        let body = build_body(&self.body())?;

        let response = client.dispatch(request, body).await?;
        update_cookies_from_response(client, &response, cookie_guard).await;

        let (parts, body) = response.into_parts();
        Ok(Response::from_parts(
            parts,
            serde_xml_rs::from_str(std::str::from_utf8(&body).unwrap())?,
        ))
    }
}

fn build_body<T>(body: &Option<T>) -> Result<Option<Vec<u8>>, QueryError>
where
    T: RequestBody,
{
    Ok(body
        .as_ref()
        .map(|body| serde_xml_rs::to_string(body))
        .transpose()?
        .map(|s| s.into_bytes()))
}

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
    let mut req = http::request::Builder::new()
        .method(E::METHOD)
        .uri(uri.as_str())
        .version(http::Version::HTTP_11);

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

async fn update_cookies_from_response<'a, S>(
    session: &'a S,
    response: &Response<Vec<u8>>,
    existing_guard: Option<MutexGuard<'a, CookieJar>>,
) where
    S: Session,
{
    let mut cookies = if let Some(lock) = existing_guard {
        lock
    } else {
        session.cookies().lock().await
    };
    if response.headers().contains_key("set-cookie") {
        let set_cookies = response.headers().get_all("set-cookie");
        cookies.set_from_multiple_headers(set_cookies);
    }
}
