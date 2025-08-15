use crate::{ContextId, ResponseBody, StatefulDispatch, StatelessDispatch};
use async_trait::async_trait;
use http::Response;
use std::borrow::Cow;

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
        return Ok(None);
    }
}

#[async_trait]
pub trait StatelessQuery<C, R> {
    async fn query(&self, client: &C) -> Result<Response<R>, String>;
}

#[async_trait]
pub trait StatefulQuery<C, R> {
    async fn query(&self, client: &C, context: ContextId) -> Result<Response<R>, String>;
}

#[async_trait]
impl<E, C> StatelessQuery<C, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    C: StatelessDispatch,
{
    async fn query(&self, client: &C) -> Result<Response<E::ResponseBody>, String> {
        let connection = client.connection();
        let uri = connection.server_url().join(&self.url()).unwrap();

        let req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        let response: http::Response<E::ResponseBody> =
            client.dispatch(req, self.body().unwrap()).await.unwrap();
        Ok(response)
    }
}

#[async_trait]
impl<E, C> StatefulQuery<C, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateful> + Sync + Send,
    C: StatefulDispatch,
{
    async fn query(
        &self,
        client: &C,
        context: ContextId,
    ) -> Result<Response<E::ResponseBody>, String> {
        if let Some(ctx) = client.context(context) {
            print!("{:?}", ctx);
        }
        let connection = client.connection();
        let uri = connection.server_url().join(&self.url()).unwrap();

        let req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        let response: http::Response<E::ResponseBody> =
            client.dispatch(req, self.body().unwrap()).await.unwrap();
        Ok(response)
    }
}
