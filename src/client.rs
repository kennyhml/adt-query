use crate::error::DispatchError;
use crate::{Context, ContextId, CookieJar, System, auth::Credentials};
use crate::{Cookie, RequestDispatch};

use arc_swap::ArcSwapOption;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::Response;
use http::request::Builder as RequestBuilder;
use std::sync::Mutex as SyncMutex;
use std::sync::atomic::{AtomicU32, Ordering};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex as AsyncMutex;

lazy_static::lazy_static! {
    /// Global context counter such that context handles are unique
    /// even across different sessions. That way, a handle from a
    /// previous session can never mistakenly be valid for a new session.
    static ref CONTEXT_COUNTER: AtomicU32 = AtomicU32::new(0);
}

/// Represents a user session on the SAP System. The session is determined
/// by the `SAP_SESSIONID_xxx` cookie. Stateful and Stateless requests
/// can both be used in the context of that same session, but the headers
/// must be managed accordingly.
#[derive(Debug)]
pub struct UserSession {
    /// Timestamp of when this session started on the backend
    start_time: DateTime<Utc>,

    /// Cookie Jar of this specific session.
    ///
    /// The `sap-contextid` cookie will not be included in this jar as it
    /// makes no sense for stateless sessions.
    cookies: AsyncMutex<CookieJar>,

    /// CSRF Token required for most POST Endpoints, bound to the session.
    csrf_token: ArcSwapOption<String>,

    /// The contexts of this session, required for stateful communication.
    ///
    /// A stateful context must, for example, be held alive for the duration
    /// an object should remain locked. For short operations that require
    /// stateful sessions, it is recommended to create a seperate context
    /// and quickly discard it otherwise to avoid needlessly busy work processes.
    contexts: SyncMutex<HashMap<ContextId, Arc<AsyncMutex<Context>>>>,
}

impl UserSession {
    fn reserve_context(&self) -> ContextId {
        let new_value = CONTEXT_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
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

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct Client<T>
where
    T: RequestDispatch,
{
    /// Request dispatch implementation, may be user defined and use either
    /// HTTP or RFC to handle the final communication with the backend system.
    dispatcher: T,

    /// The SAP System this client is connecting / connected with.
    system: System,

    /// The client number that we are connecting / connected to the SAP System with.
    client: i32,

    #[builder(setter(skip))]
    session: Option<UserSession>,

    #[builder(setter(into))]
    language: String,

    credentials: Credentials,

    /// Number of requests this client has dispatched
    #[builder(setter(skip), default = 0)]
    dispatch_count: i32,
}

impl<T> Client<T>
where
    T: RequestDispatch,
{
    // async fn inject_request_context<'a, S>(
    //     headers: &mut HeaderMap<HeaderValue>,
    //     session: &'a S,
    //     context_id: ContextId,
    // ) -> Result<(), QueryError>
    // where
    //     S: Contextualize,
    // {
    //     headers.insert(
    //         "x-sap-adt-sessiontype",
    //         HeaderValue::from_static("stateful"),
    //     );

    //     if let Some(context_data) = session.context(context_id) {
    //         let cookie_header = headers
    //             .iter_mut()
    //             .find(|(k, _)| *k == "cookie")
    //             .map(|(_, v)| v);

    //         if let Some(value) = cookie_header {
    //             let as_string = value.to_str().unwrap();
    //             *value = HeaderValue::from_str(&format!(
    //                 "{as_string}; {}",
    //                 context_data.lock().await.cookie().as_cookie_pair()
    //             ))
    //             .unwrap();
    //         } else {
    //             return Err(QueryError::CookiesMissing);
    //         }
    //     } else {
    //         // Context has not yet been created on the server, in this case
    //         // setting the sessiontype header is good enough.
    //     }
    //     Ok(())
    // }

    pub async fn dispatch_stateless(
        &self,
        request: RequestBuilder,
        body: Vec<u8>,
    ) -> Result<Response<Vec<u8>>, DispatchError> {
        // Prepare request
        self.dispatcher.dispatch_request(request, body).await
        // Handle response cookies
    }

    pub async fn dispatch_stateful(
        &self,
        request: RequestBuilder,
        body: Vec<u8>,
        context: ContextId,
    ) -> Result<Response<Vec<u8>>, DispatchError> {
        // Prepare request
        self.dispatcher.dispatch_request(request, body).await
        // Handle response cookies
    }

    pub fn destination(&self) -> &System {
        &self.system
    }

    pub fn client(&self) -> i32 {
        self.client
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}

#[async_trait]
impl RequestDispatch for reqwest::Client {
    async fn dispatch_request(
        &self,
        request: RequestBuilder,
        body: Vec<u8>,
    ) -> Result<Response<Vec<u8>>, DispatchError> {
        todo!()
        // let request = request.body(body)?;

        // let response = self
        //     .request(request.method(), request.uri().to_string())
        //     .body(request.body().clone())
        //     .headers(request.headers().clone())
        //     .send();

        // let mut mapped = Response::builder().status(response.status());
        // if let Some(headers) = mapped.headers_mut() {
        //     *headers = response.headers().clone();
        // }
        // Ok(mapped.body(response.text().await?.into())?)
    }
}

/// Updates the session cookies from the `set-cookie` headers in the response.
///
/// After this step is done, this is also where the cookie jar mutex guard will
/// be dropped in any case and is available for concurrent access going forward.
// async fn update_cookies_from_response<'a, S>(
//     session: &'a S,
//     response_headers: &HeaderMap,
//     existing_guard: Option<MutexGuard<'a, CookieJar>>,
// ) where
//     S: Session,
// {
//     if let Some(csrf_token) = response_headers.get("x-csrf-token") {
//         session
//             .csrf_token()
//             .store(Some(Arc::new(csrf_token.to_str().unwrap().to_owned())));
//     }

//     // No cookies to update, avoid locking the jar where possible.
//     if !response_headers.contains_key("set-cookie") {
//         return;
//     }
//     let mut cookies = match existing_guard {
//         Some(lock) => lock,
//         None => session.cookies().lock().await,
//     };

//     cookies.set_from_multiple_headers(response_headers.get_all("set-cookie"));
// }

/// Updates the session cookies from the `set-cookie` headers in the response.
///
/// After this step is done, this is also where the cookie jar mutex guard will
/// be dropped in any case and is available for concurrent access going forward.
// async fn update_session_from_response<'a, S>(
//     session: &'a S,
//     response_headers: &HeaderMap,
//     existing_guard: Option<MutexGuard<'a, CookieJar>>,
//     context: ContextId,
// ) where
//     S: Session,
// {
//     if let Some(csrf_token) = response_headers.get("x-csrf-token") {
//         session
//             .csrf_token()
//             .store(Some(Arc::new(csrf_token.to_str().unwrap().to_owned())));
//     }

//     // No cookies to update, avoid locking the jar where possible.
//     if !response_headers.contains_key("set-cookie") {
//         return;
//     }
//     let mut cookies = match existing_guard {
//         Some(lock) => lock,
//         None => session.cookies().lock().await,
//     };

//     cookies.set_from_multiple_headers(response_headers.get_all("set-cookie"));
//     if let Some(session_cookie) = cookies.take("sap-contextid") {
//         match session.context(context) {
//             Some(context) => context.lock().await.update(session_cookie),
//             None => session.insert_context(context, session_cookie),
//         }
//     }
// }

#[cfg(test)]
pub mod tests {
    // use std::collections::HashSet;
    // use std::str::FromStr as _;

    // use std::sync::{Arc, Mutex};
    // use std::thread;
    // use url::Url;

    // use crate::SystemBuilder;

    // use super::*;

    // fn test_client() -> Client {
    //     let system = SystemBuilder::default()
    //         .name("A4H")
    //         .server_url(Url::from_str("http://localhost:50000").unwrap())
    //         .build()
    //         .unwrap();

    //     ClientBuilder::default()
    //         .system(system)
    //         .language("en")
    //         .client(001)
    //         .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
    //         .build()
    //         .unwrap()
    // }

    // #[test]
    // fn distinct_contexts_get_created() {
    //     let client = test_client();

    //     let first_contex = client.create_context();
    //     let second_context = client.create_context();

    //     assert_ne!(
    //         first_contex, second_context,
    //         "Context identifiers are not unique."
    //     );
    // }

    // #[tokio::test]
    // async fn context_gets_inserted() {
    //     let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

    //     let client = test_client();

    //     let context_id = client.create_context();
    //     client.insert_context(context_id, Cookie::parse(cookie).unwrap());

    //     let ctx = client.context(context_id);
    //     assert!(ctx.is_some(), "Context was not inserted");
    //     assert_eq!(
    //         ctx.unwrap().lock().await.cookie().value(),
    //         "SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT"
    //     )
    // }

    // #[tokio::test]
    // async fn context_gets_dropped() {
    //     let cookie = "sap-contextid=SID%3aANON%3avhcala4hci_A4H_00%3aBx0ChjXcVBx8y7eJra9fIFMVL6IIu-Z7PJLU-Mvc-ATT; path=/sap/bc/adt";

    //     let client = test_client();

    //     let context_id = client.create_context();
    //     client.insert_context(context_id, Cookie::parse(cookie).unwrap());
    //     client.drop_context(context_id);

    //     let ctx = client.context(context_id);

    //     assert!(ctx.is_none(), "Context was not dropped");
    // }

    // #[test]
    // fn context_reservation_is_thread_safe() {
    //     let client = Arc::new(Mutex::new(test_client()));
    //     let contexts = Arc::new(Mutex::new(vec![]));
    //     let mut handles = vec![];

    //     for _ in 0..10 {
    //         let client = Arc::clone(&client);
    //         let contexts = Arc::clone(&contexts);
    //         let handle = thread::spawn(move || {
    //             let context = client.lock().unwrap().create_context();
    //             contexts.lock().unwrap().push(context);
    //         });
    //         handles.push(handle);
    //     }

    //     // Wait for all threads to complete
    //     for handle in handles {
    //         handle.join().unwrap();
    //     }

    //     let set: HashSet<_> = contexts.lock().unwrap().drain(..).collect();
    //     assert_eq!(set.len(), 10, "Not all context ids are unique.");
    // }
}
