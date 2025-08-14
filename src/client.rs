use std::{borrow::Cow, error::Error, fmt::Debug};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::Response;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::{
    common::{Cookie, CookieJar},
    system::ConnectionConfiguration,
};

trait State {}

pub trait ContextStore {
    fn get_context(&self) -> &Context {
        todo!()
    }

    fn drop_contex(&self) -> Context {
        todo!()
    }
}

#[async_trait]
pub trait RequestDispatch {
    async fn dispatch<R>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<R>, String>
    where
        R: DeserializeOwned + Send + Debug,
    {
        todo!()
    }
}

pub trait StatefulDispatch: RequestDispatch + ContextStore + Sync + Send {}
pub trait StatelessDispatch: RequestDispatch + Sync + Send {}

pub type ContextId = u32;
// Represents a context within a session
pub struct Context {
    // ID of the context, serves as internal handle to the context.
    _id: ContextId,
    created: DateTime<Utc>,
    context_id: Cookie,

    requests_made: i32,
}

struct Connected {
    started: DateTime<Utc>,
    session_id: Cookie,
}

impl State for () {}
impl State for Connected {}

pub struct Client<S: State> {
    config: ConnectionConfiguration,

    state: S,
}

impl Client<()> {
    async fn connect(self) -> Result<Client<Connected>, (Self, String)> {
        todo!()
    }
}

impl Client<Connected> {
    async fn disconnect(self) -> Client<()> {
        todo!()
    }
}

// #[cfg(feature = "reqwest")]
// #[async_trait]
// impl HTTPClient for reqwest::Client {
//     async fn get<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
//     where
//         R: Send + Sync + DeserializeOwned,
//     {
//         let response = self
//             .get(request.uri().to_string())
//             .body(request.body().clone())
//             .headers(request.headers().clone())
//             .send()
//             .await
//             .unwrap();
//         let mut mapped = Response::builder().status(response.status());
//         if let Some(headers) = mapped.headers_mut() {
//             *headers = response.headers().clone();
//         }

//         mapped
//             .body(serde_xml_rs::from_str(&response.text().await.unwrap()).unwrap())
//             .unwrap()
//     }

//     async fn post<R>(&self, request: &Request<Vec<u8>>) -> Response<R>
//     where
//         R: Send + Sync + DeserializeOwned,
//     {
//         todo!()
//     }
// }
