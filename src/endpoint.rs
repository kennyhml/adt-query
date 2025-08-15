use crate::{ContextId, StatefulDispatch, StatelessDispatch};
use async_trait::async_trait;
use http::Response;
use serde::de::DeserializeOwned;
use std::{borrow::Cow, fmt::Debug};

pub trait EndpointKind {}

pub struct Stateless {}
pub struct Stateful {}

impl EndpointKind for Stateful {}
impl EndpointKind for Stateless {}

pub trait Endpoint {
    type ResponseBody: DeserializeOwned + Sync + Send + Debug;
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
        todo!()
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
        todo!()
    }
}
