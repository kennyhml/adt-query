use crate::common::Cookie;
use crate::{
    ClientNumber, Context, ContextId, Contextualize, Session, System, auth::Credentials,
    common::CookieJar, error::QueryError,
};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::{Response, request::Builder as RequestBuilder};
use std::sync::Mutex as SyncMutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Builder, Debug)]
pub struct Client {
    system: System,

    #[builder(default=reqwest::Client::new())]
    http_client: reqwest::Client,

    #[builder(setter(skip), default=None)]
    start: Option<DateTime<Utc>>,

    #[builder(setter(skip), default=Arc::new(AsyncMutex::new(CookieJar::new())))]
    cookies: Arc<AsyncMutex<CookieJar>>,

    #[builder(setter(skip), default = std::sync::Mutex::new(HashMap::new()))]
    contexts: SyncMutex<HashMap<ContextId, Arc<AsyncMutex<Context>>>>,

    #[builder(setter(skip), default = AtomicU32::new(0))]
    context_counter: AtomicU32,

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
    fn reserve_context(&self) -> ContextId {
        let new_value = self.context_counter.fetch_add(1, Ordering::SeqCst) + 1;
        ContextId(new_value)
    }

    fn insert_context(&self, id: ContextId, cookie: Cookie) {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(id, Arc::new(AsyncMutex::new(Context::new(id, cookie))));
    }

    fn context(&self, id: ContextId) -> Option<Arc<AsyncMutex<Context>>> {
        let contexts = self.contexts.lock().unwrap();
        contexts.get(&id).cloned()
    }

    fn drop_context(&self, id: ContextId) -> Option<Arc<AsyncMutex<Context>>> {
        self.contexts.lock().unwrap().remove(&id)
    }
}

#[async_trait]
impl Session for Client {
    async fn dispatch(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, QueryError> {
        let request = request.body(body.unwrap_or_default())?;

        let response = self
            .http_client
            .get(request.uri().to_string())
            .body(request.body().clone())
            .headers(request.headers().clone())
            .send()
            .await?;

        if response.status() == 401 {
            return Err(QueryError::Unauthorized);
        }

        //TOOD: Other status codes can also be ok/expected (such as 304 not modified)
        if response.status() != 200 {
            return Err(QueryError::BadStatusCode {
                code: response.status(),
                message: "".to_owned(),
            });
        }

        let mut mapped = Response::builder().status(response.status());
        if let Some(headers) = mapped.headers_mut() {
            *headers = response.headers().clone();
        }
        Ok(mapped.body(response.bytes().await?.into())?)
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

    fn cookies(&self) -> &Arc<AsyncMutex<CookieJar>> {
        &self.cookies
    }

    fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}
