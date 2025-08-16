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
pub trait StatelessQuery<T, R> {
    async fn query(&self, session: &T) -> Result<Response<R>, String>;
}

#[async_trait]
pub trait StatefulQuery<T, R> {
    async fn query(&self, session: &T, context: ContextId) -> Result<Response<R>, String>;
}

#[async_trait]
impl<E, T> StatelessQuery<T, E::ResponseBody> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    T: StatelessDispatch,
{
    async fn query(&self, session: &T) -> Result<Response<E::ResponseBody>, String> {
        let uri = session.base_url().join(&self.url()).unwrap();

        let req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        let response: http::Response<E::ResponseBody> =
            session.dispatch(req, self.body().unwrap()).await.unwrap();
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
        session: &T,
        context: ContextId,
    ) -> Result<Response<E::ResponseBody>, String> {
        if let Some(ctx) = session.context(context) {
            print!("{:?}", ctx);
        }
        let uri = session.base_url().join(&self.url()).unwrap();

        let req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str());

        let response: http::Response<E::ResponseBody> =
            session.dispatch(req, self.body().unwrap()).await.unwrap();
        Ok(response)
    }
}
