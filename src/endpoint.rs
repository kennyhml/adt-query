use crate::{
    ContextId, RequestBody, ResponseBody, StatefulDispatch, StatelessDispatch, common::CookieJar,
    error::QueryError,
};
use async_trait::async_trait;
use http::{HeaderMap, Request, Response};
use std::{borrow::Cow, sync::Arc};
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

        let destination = client.destination();
        let uri = destination.server_url().join(&self.url())?;
        let mut req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        if let Some(headers) = self.headers() {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }
        let cookie_mutex = client.cookies();
        let cookies = cookie_mutex.lock().await;

        let cookie_lock: Option<MutexGuard<'_, CookieJar>> = if cookies.is_empty() {
            req = req.header("Authorization", client.credentials().basic_auth());
            Some(cookies)
        } else {
            req = req.header("Cookie", cookies.to_header(&uri)?);
            drop(cookies);
            None
        };

        let body = self
            .body()
            .map(|body| serde_xml_rs::to_string(&body))
            .transpose()?
            .map(|s| s.into_bytes());

        let response = client.dispatch(req, body).await?;
        {
            let mut cookies = if let Some(lock) = cookie_lock {
                lock
            } else {
                cookie_mutex.lock().await
            };
            if response.headers().contains_key("set-cookie") {
                let set_cookies = response.headers().get_all("set-cookie");
                cookies.set_from_multiple_headers(set_cookies);
            }
        }
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
        todo!()
    }
}
