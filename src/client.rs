use std::{borrow::Cow, fmt::Debug};

use async_trait::async_trait;
use http::{Request, Response, Uri};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_xml_rs;
use url::Url;

use http::request::Builder as RequestBuilder;

use crate::{http::HTTPClient, system::ConnectionConfiguration};

pub trait SessionState {}

pub struct Connected {
    sso_token: String,
    session_id: String,
    usercontext: String,
    context_id: String,
}

impl SessionState for () {}
impl SessionState for Connected {}

#[async_trait]
pub trait RestClient {
    async fn request<R>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<R>, String>
    where
        R: DeserializeOwned + Sync + Send + Debug;

    fn server_url<'a>(&'a self) -> Cow<'a, Url>;
}

#[derive(Debug)]
pub struct Client<T: HTTPClient, S: SessionState> {
    // The Client that drives the HTTP Communication
    http_client: T,

    connection_config: ConnectionConfiguration,

    session_state: S,
}

impl<T: HTTPClient> Client<T, ()> {
    pub fn new(http_client: T, config: ConnectionConfiguration) -> Self {
        Self {
            http_client,
            connection_config: config,
            session_state: (),
        }
    }

    pub async fn login(self) -> Result<Client<T, Connected>, (String, Self)> {
        Ok(Client {
            http_client: self.http_client,
            connection_config: self.connection_config,
            session_state: Connected {
                sso_token: String::new(),
                session_id: String::new(),
                usercontext: String::new(),
                context_id: String::new(),
            },
        })
    }
}

impl<T: HTTPClient> Client<T, Connected> {
    async fn logout(self) -> Result<Client<T, ()>, (String, Self)> {
        todo!()
    }
}

#[async_trait]
impl<C: HTTPClient> RestClient for Client<C, Connected> {
    async fn request<R>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<R>, String>
    where
        R: DeserializeOwned + Sync + Send + Debug,
    {
        let req = request.body(body.unwrap_or_default()).unwrap();
        let response: Response<R> = self.http_client.get(&req).await;
        Ok(response)
    }

    fn server_url<'a>(&'a self) -> Cow<'a, Url> {
        Cow::Borrowed(self.connection_config.server_url())
    }
}

#[cfg(feature = "reqwest")]
#[async_trait]
impl HTTPClient for reqwest::Client {
    async fn get<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
    where
        R: Send + Sync + DeserializeOwned,
    {
        let response = self
            .get(request.uri().to_string())
            .body(request.body().clone())
            .headers(request.headers().clone())
            .send()
            .await
            .unwrap();
        let mut mapped = Response::builder().status(response.status());
        if let Some(headers) = mapped.headers_mut() {
            *headers = response.headers().clone();
        }

        mapped
            .body(serde_xml_rs::from_str(&response.text().await.unwrap()).unwrap())
            .unwrap()
    }

    async fn post<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
    where
        R: Send + Sync + DeserializeOwned,
    {
        todo!()
    }
}
