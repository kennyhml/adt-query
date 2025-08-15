use crate::{RequestDispatch, ResponseBody, common::Cookie, system::ConnectionConfiguration};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::{Response, request::Builder as RequestBuilder};

pub trait State {}

pub struct ConnectedState {
    connected: DateTime<Utc>,
    session_id: Cookie,
}

impl State for () {}
impl State for ConnectedState {}

pub struct Client<S: State> {
    config: ConnectionConfiguration,
    http_client: reqwest::Client,

    state: S,
}

impl Client<()> {
    async fn connect(self) -> Result<Client<ConnectedState>, (Self, String)> {
        todo!()
    }
}

impl Client<ConnectedState> {
    async fn disconnect(self) -> Client<()> {
        todo!()
    }
}

#[async_trait]
impl RequestDispatch for Client<ConnectedState> {
    async fn dispatch<T>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<T>, String>
    where
        T: ResponseBody,
    {
        // This would be the place to set the headers for the request
        // such as session headers, user context, etc..
        // DONT set the stateful headers though, the stateful query should do that!
        let request = request.body(body.unwrap_or_default()).unwrap();

        let response = self
            .http_client
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

        Ok(mapped
            .body(serde_xml_rs::from_str(&response.text().await.unwrap()).unwrap())
            .unwrap())
    }

    fn connection(&self) -> &ConnectionConfiguration {
        &self.config
    }
}
