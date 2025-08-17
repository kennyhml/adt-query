use std::{borrow::Cow, collections::HashMap};

use crate::{
    Context, ContextId, Contextualize, RequestDispatch, ResponseBody, System, auth::Credentials,
    common::Cookie,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::{Response, request::Builder as RequestBuilder};
use url::Url;

#[derive(Builder, Debug)]
pub struct Client {
    system: System,

    #[builder(default=reqwest::Client::new())]
    http_client: reqwest::Client,

    #[builder(setter(skip), default=None)]
    start: Option<DateTime<Utc>>,

    #[builder(setter(skip), default=None)]
    session_id: Option<Cookie>,

    #[builder(setter(skip), default = HashMap::new())]
    contexts: HashMap<ContextId, Option<Context>>,

    #[builder(setter(skip), default = 0)]
    context_counter: u32,

    // The client to connect on
    client: i32,

    // The language to connect with, e.g 'EN', 'DE'..
    #[builder(setter(into))]
    language: String,

    credentials: Credentials,
}

impl Contextualize for Client {
    fn context(&self, id: ContextId) -> Option<&Context> {
        self.contexts.get(&id).and_then(|opt| opt.as_ref())
    }

    fn new_context(&mut self) -> ContextId {
        self.context_counter += 1;
        ContextId(self.context_counter)
    }

    fn drop_context(&mut self, id: ContextId) -> Option<Context> {
        self.contexts.remove(&id)?
    }
}

#[async_trait]
impl RequestDispatch for Client {
    async fn dispatch<T>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<T>, String>
    where
        T: ResponseBody,
    {
        // This would be the place to set the headers for the request
        // such as Client headers, user context, etc..
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

    fn base_url(&self) -> Cow<'_, Url> {
        self.system.server_url()
    }
}
