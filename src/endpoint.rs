use crate::{ContextId, ResponseBody, StatefulDispatch, StatelessDispatch};
use async_trait::async_trait;
use http::{HeaderMap, Response};
use std::borrow::Cow;
use tracing::{Level, event, instrument};

pub trait EndpointKind {}

pub struct Stateless {}
pub struct Stateful {}

impl EndpointKind for Stateful {}
impl EndpointKind for Stateless {}

pub trait Endpoint {
    type ResponseBody: ResponseBody;
    type Kind: EndpointKind;

    const METHOD: http::Method;

    fn url(&self) -> Cow<'static, str>;

    fn body(&self) -> Result<Option<Vec<u8>>, String> {
        Ok(None)
    }

    fn headers(&self) -> Option<&HeaderMap> {
        None
    }
}

#[async_trait]
pub trait StatelessQuery<T, R> {
    async fn query(&self, client: &T) -> Result<Response<R>, String>;
}

#[async_trait]
pub trait StatefulQuery<T, R> {
    async fn query(&self, client: &T, context: ContextId) -> Result<Response<R>, String>;
}

#[async_trait]
impl<E, T> StatelessQuery<T, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    T: StatelessDispatch,
{
    #[instrument(skip(self, client), level = Level::INFO, fields(system = client.destination().server_url().to_string()))]
    async fn query(&self, client: &T) -> Result<Response<E::ResponseBody>, String> {
        let destination = client.destination();
        let uri = destination.server_url().join(&self.url()).unwrap();

        let mut req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        if let Some(headers) = self.headers() {
            for (k, v) in headers {
                req = req.header(k, v);
            }
        }

        let cookies = client.cookies().lock().await.to_header(&uri).unwrap();
        if !cookies.is_empty() {
            event!(Level::DEBUG, "Reusing session cookies.");
            req = req.header("Cookie", cookies);
        } else {
            println!("Using basic auth...");
            req = req.header("Authorization", client.credentials().basic_auth());
        }

        let response: http::Response<E::ResponseBody> =
            client.dispatch(req, self.body().unwrap()).await.unwrap();

        let set_cookies = response.headers().get_all("set-cookie");
        println!("Setting cookies: {:?}", set_cookies);
        client
            .cookies()
            .lock()
            .await
            .set_from_multiple_headers(set_cookies);
        Ok(response)
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
    ) -> Result<Response<E::ResponseBody>, String> {
        todo!()
    }
}
