use std::collections::HashMap;

use crate::{
    Context, ContextId, Contextualize, RequestDispatch, ResponseBody, common::Cookie,
    system::ConnectionConfiguration,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::{Response, request::Builder as RequestBuilder};

pub trait State {}
impl State for () {}
impl State for ConnectedState {}

pub struct ConnectedState {
    connected: DateTime<Utc>,
    session_id: Cookie,

    contexts: HashMap<ContextId, Option<Context>>,
    context_counter: u32,
}

#[derive(Debug)]
pub struct Client<S: State> {
    config: ConnectionConfiguration,
    http_client: reqwest::Client,

    state: S,
}

impl Client<()> {
    pub fn new(config: ConnectionConfiguration) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            state: (),
        }
    }

    pub async fn connect(self) -> Result<Client<ConnectedState>, (Self, String)> {
        // This is going to need improving. Implementing the Dispatch trait for this state
        // would allow the end user to try to use a client that isnt connected. While that
        // still works (at least as long as you dont need a context) it would end up creating
        // a new session on the server for every request.. (see in SM05)
        todo!()
    }
}

impl Client<ConnectedState> {
    pub async fn disconnect(self) -> Client<()> {
        // Similar problem as with the connect endpoint. These endpoints would essentially
        // be endpoints that modify the state of the client. I.e the logoff endpoint cannot
        // really be called with a connected client because the client will disconnect when you do.

        // Basically this also means we should just implement the "endpoint" trait for these.
        todo!()
    }
}

impl Contextualize for Client<ConnectedState> {
    fn context(&self, id: ContextId) -> Option<&Context> {
        self.state.contexts.get(&id).and_then(|opt| opt.as_ref())
    }

    fn new_context(&mut self) -> ContextId {
        self.state.context_counter += 1;
        ContextId(self.state.context_counter)
    }

    fn drop_context(&mut self, id: ContextId) -> Option<Context> {
        self.state.contexts.remove(&id)?
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
