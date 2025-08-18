use crate::{
    ClientNumber, Context, ContextId, Contextualize, ResponseBody, Session, System,
    auth::Credentials, common::CookieJar,
};
use arc_swap::ArcSwap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::{Response, request::Builder as RequestBuilder};
use std::{collections::HashMap, sync::Arc};

#[derive(Builder, Debug)]
pub struct Client {
    system: System,

    #[builder(default=reqwest::Client::new())]
    http_client: reqwest::Client,

    #[builder(setter(skip), default=None)]
    start: Option<DateTime<Utc>>,

    #[builder(setter(skip), default=ArcSwap::new(Arc::new(CookieJar::new())))]
    cookies: ArcSwap<CookieJar>,

    #[builder(setter(skip), default = HashMap::new())]
    contexts: HashMap<ContextId, Option<Context>>,

    #[builder(setter(skip), default = 0)]
    context_counter: u32,

    // The client to connect on
    #[builder(setter(into))]
    client: ClientNumber,

    // The language to connect with, e.g 'EN', 'DE'..
    #[builder(setter(into))]
    language: String,

    #[builder(setter(skip), default = false)]
    authenticated: bool,

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
impl Session for Client {
    async fn dispatch<T>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<T>, String>
    where
        T: ResponseBody,
    {
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

    fn destination(&self) -> &System {
        &self.system
    }

    fn client(&self) -> ClientNumber {
        self.client
    }

    fn language(&self) -> &str {
        &self.language
    }

    fn cookies(&self) -> &ArcSwap<CookieJar> {
        &self.cookies
    }

    fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}
