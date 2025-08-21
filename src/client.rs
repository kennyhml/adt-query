use crate::Cookie;
use crate::{
    ClientNumber, Context, ContextId, Contextualize, CookieJar, Session, System, auth::Credentials,
    error::QueryError,
};

use arc_swap::ArcSwapOption;
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

    #[builder(setter(skip), default=ArcSwapOption::new(None))]
    csrf_token: ArcSwapOption<String>,

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
        body: Option<String>,
    ) -> Result<Response<String>, QueryError> {
        let request = request.body(body.unwrap_or_default())?;

        println!("{:?}", request);
        let response = self
            .http_client
            .request(request.method().clone(), request.uri().to_string())
            .body(request.body().clone())
            .headers(request.headers().clone())
            .send()
            .await?;

        let mut mapped = Response::builder().status(response.status());
        if let Some(headers) = mapped.headers_mut() {
            *headers = response.headers().clone();
        }
        Ok(mapped.body(response.text().await?.into())?)
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

    fn csrf_token(&self) -> &ArcSwapOption<String> {
        &self.csrf_token
    }

    fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

#[cfg(test)]
pub mod tests {
    use std::collections::HashSet;
    use std::str::FromStr as _;

    use std::sync::{Arc, Mutex};
    use std::thread;
    use url::Url;

    use crate::SystemBuilder;

    use super::*;

    fn test_client() -> Client {
        let system = SystemBuilder::default()
            .name("A4H")
            .server_url(Url::from_str("http://localhost:50000").unwrap())
            .build()
            .unwrap();

        ClientBuilder::default()
            .system(system)
            .language("en")
            .client(001)
            .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
            .build()
            .unwrap()
    }

    #[test]
    fn distinct_contexts_get_created() {
        let client = test_client();

        let first_contex = client.reserve_context();
        let second_context = client.reserve_context();

        assert_ne!(
            first_contex, second_context,
            "Context identifiers are not unique."
        );
    }

    #[tokio::test]
    async fn context_gets_inserted() {
        let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

        let client = test_client();

        let context_id = client.reserve_context();
        client.insert_context(context_id, Cookie::parse(cookie).unwrap());

        let ctx = client.context(context_id);
        assert!(ctx.is_some(), "Context was not inserted");
        assert_eq!(
            ctx.unwrap().lock().await.cookie().value(),
            "SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT"
        )
    }

    #[tokio::test]
    async fn context_gets_dropped() {
        let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

        let client = test_client();

        let context_id = client.reserve_context();
        client.insert_context(context_id, Cookie::parse(cookie).unwrap());
        client.drop_context(context_id);

        let ctx = client.context(context_id);

        assert!(ctx.is_none(), "Context was not dropped");
    }

    #[test]
    fn context_reservation_is_thread_safe() {
        let client = Arc::new(Mutex::new(test_client()));
        let contexts = Arc::new(Mutex::new(vec![]));
        let mut handles = vec![];

        for _ in 0..10 {
            let client = Arc::clone(&client);
            let contexts = Arc::clone(&contexts);
            let handle = thread::spawn(move || {
                let context = client.lock().unwrap().reserve_context();
                contexts.lock().unwrap().push(context);
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        let set: HashSet<_> = contexts.lock().unwrap().drain(..).collect();
        assert_eq!(set.len(), 10, "Not all context ids are unique.");
    }
}
