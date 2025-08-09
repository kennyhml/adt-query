use async_trait::async_trait;
use http::Response;
use serde::de::DeserializeOwned;
use std::{borrow::Cow, fmt::Debug};

use crate::client::RestClient;

pub trait Endpoint {
    type ResponseBody: DeserializeOwned + Sync + Send + Debug;

    const STATEFUL: bool;
    const METHOD: http::Method;

    fn url(&self) -> Cow<'static, str>;

    fn body(&self) -> Result<Option<Vec<u8>>, String> {
        return Ok(None);
    }
}

#[async_trait]
pub trait Query<C, R> {
    async fn query(&self, client: &C) -> Result<Response<R>, String>;
}

#[async_trait]
impl<E, C> Query<C, E::ResponseBody> for E
where
    E: Endpoint + Sync + Send,
    C: RestClient + Sync + Send,
{
    async fn query(&self, client: &C) -> Result<Response<E::ResponseBody>, String> {
        let uri = client.config().server_url().join(&self.url()).unwrap();

        let req = http::request::Builder::new()
            .method(Self::METHOD)
            .uri(uri.as_str())
            .header("Authorization", "Basic REVWRUxPUEVSOkFCQVB0cjIwMjIjMDE=")
            .header("Accept", "*/*")
            .header("Accept-Encoding", "gzip, deflate, br");

        let response: http::Response<E::ResponseBody> =
            client.request(req, self.body().unwrap()).await.unwrap();
        Ok(response)
    }
}
