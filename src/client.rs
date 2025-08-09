use std::fmt::Debug;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderValue, Request, Response, header::GetAll};
use secrecy::SecretString;
use serde::de::DeserializeOwned;
use serde_xml_rs;

use http::request::Builder as RequestBuilder;

use crate::{
    api::common::{Cookie, Header},
    system::ConnectionConfiguration,
};

pub trait ClientState {}

#[derive(Debug)]
pub struct Connected {
    start: DateTime<Utc>,

    sso2_token: Option<String>,
    session_id: String,
    usercontext: String,
    context_id: Option<String>,
}

impl ClientState for () {}
impl ClientState for Connected {}

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

#[async_trait]
pub trait RestClient {
    async fn request<R>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<R>, String>
    where
        R: DeserializeOwned + Sync + Send + Debug;

    fn config(&self) -> &ConnectionConfiguration;

    fn connection(&self) -> &Connected;
}

#[derive(Debug)]
pub struct Client<T: HTTPClient, S: ClientState> {
    // The Client that drives the HTTP Communication
    http_client: T,

    config: ConnectionConfiguration,

    state: S,
}

impl<T: HTTPClient> Client<T, ()> {
    pub fn new(http_client: T, config: ConnectionConfiguration) -> Self {
        Self {
            http_client,
            config: config,
            state: (),
        }
    }

    pub async fn login(self) -> Result<Client<T, Connected>, (String, Self)> {
        let request = RequestBuilder::new()
            .uri(
                self.config
                    .server_url()
                    .join("/sap/bc/adt/core/discovery")
                    .unwrap()
                    .as_str(),
            )
            .header("Authorization", self.config.credentials().basic_auth())
            .header("x-crsf-token", "fetch")
            .body(Vec::new())
            .unwrap();

        let res: Response<()> = self.http_client.get(&request).await;
        let cookies = res.headers().get_all("set-cookie");
        let state = Connected::parse(&cookies);
        println!("{state:?}");

        Ok(Client {
            http_client: self.http_client,
            config: self.config,
            state,
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

    fn config(&self) -> &ConnectionConfiguration {
        &self.config
    }

    fn connection(&self) -> &Connected {
        &self.state
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

impl Connected {
    fn parse(cookies: &GetAll<'_, HeaderValue>) -> Self {
        let sso2 = cookies.iter().find_map(|v| {
            v.to_str()
                .ok()
                .filter(|v| v.contains(Cookie::SSO2))
                .and_then(|v| Some(v.to_owned()))
        });

        let session_id = cookies
            .iter()
            .find_map(|v| {
                v.to_str()
                    .ok()
                    .filter(|v| v.contains(Cookie::SAP_SESSIONID))
                    .and_then(|v| Some(v.to_owned()))
            })
            .unwrap();

        let user_context = cookies
            .iter()
            .find_map(|v| {
                v.to_str()
                    .ok()
                    .filter(|v| v.contains(Cookie::USER_CONTEXT))
                    .and_then(|v| Some(v.to_owned()))
            })
            .unwrap();

        let context_id = cookies.iter().find_map(|v| {
            v.to_str()
                .ok()
                .filter(|v| v.contains(Cookie::SAP_CONTEXT_ID))
                .and_then(|v| Some(v.to_owned()))
        });

        Self {
            start: Utc::now(),
            sso2_token: sso2,
            session_id: session_id,
            usercontext: user_context,
            context_id: context_id,
        }
    }
}
