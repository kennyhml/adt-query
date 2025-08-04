use http::{Request, Response};
use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use derive_builder::Builder;

use crate::auth::AuthorizationKind;

/// The interface that any HTTP Client must implement for use in the ADT Client.
#[async_trait]
pub trait HTTPClient: Send + Sync {
    async fn get<T: Send>(&self, options: Request<T>) -> Response<T>;

    async fn post<T: Send>(&self, options: Request<T>) -> Response<T>;
}
