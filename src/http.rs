use std::{collections::HashMap, time::Duration};

use async_trait::async_trait;
use derive_builder::Builder;

use crate::auth::AuthorizationKind;

/// The interface that any HTTP Client must implement for use in the ADT Client.
#[async_trait]
pub trait HTTPClient: Send + Sync {
    type ResponseType: Response;

    async fn get(url: &str, options: Request) -> Self::ResponseType;

    async fn post(url: &str, options: Request) -> Self::ResponseType;
}

/// HTTP Response a HTTP Client must return in order to be compatible with the ADT Client.
pub trait Response {
    fn status_code(&self) -> i32;

    fn headers(&self) -> &HashMap<String, String>;

    fn body(&self) -> &str;

    fn url(&self) -> &str;
}

/// Request Options for a HTTP Request to the ADT Service
#[derive(Default, Builder, Debug)]
#[builder(setter(into))]
pub struct Request {
    #[builder(private)]
    headers: HashMap<String, String>,
    #[builder(private, default)]
    query_parameters: HashMap<String, String>,
    #[builder(private, default = None)]
    auth: Option<AuthorizationKind>,
    #[builder(private, default = None)]
    body: Option<String>,
    #[builder(private, default = None)]
    timeout: Option<Duration>,
}

impl RequestBuilder {
    pub fn header<T: Into<String>>(&mut self, key: T, value: T) -> &mut Self {
        self.headers
            .get_or_insert_default()
            .insert(key.into(), value.into());
        self
    }

    pub fn query_parameter<K: Into<String>, V: ToString>(&mut self, key: K, value: V) -> &mut Self {
        self.query_parameters
            .get_or_insert_default()
            .insert(key.into(), value.to_string());
        self
    }
}
