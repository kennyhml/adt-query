use async_trait::async_trait;
use http::{Request, Response};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// The interface that any HTTP Client must implement for use in the ADT Client.
#[async_trait]
pub trait HTTPClient: Send + Sync {
    async fn get<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
    where
        R: Send + Sync + DeserializeOwned;

    async fn post<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
    where
        R: Send + Sync + DeserializeOwned;
}
