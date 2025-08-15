use chrono::{DateTime, Utc};

use crate::{common::Cookie, system::ConnectionConfiguration};

pub trait State {}

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
