use async_trait::async_trait;
use http::{Request, Response};
use reqwest::get;

use crate::{auth::Credentials, http::HTTPClient};

pub trait SessionState {}

pub struct Connected {
    session_id: String,
}

impl SessionState for () {}
impl SessionState for Connected {}

pub struct Client<T: HTTPClient, S: SessionState> {
    // The Client that drives the HTTP Communication
    http_client: T,

    sap_server: SAPServer,

    session_state: S,
}

impl<T: HTTPClient> Client<T, ()> {
    async fn login(self) -> Result<Client<T, Connected>, (String, Self)> {
        todo!()
    }
}

impl<T: HTTPClient> Client<T, Connected> {
    async fn logout(self) -> Result<Client<T, ()>, (String, Self)> {
        todo!()
    }
}

#[async_trait]
impl HTTPClient for reqwest::Client {
    async fn get<T: Send>(&self, options: Request<T>) -> Response<T> {
        todo!()
    }

    async fn post<T: Send>(&self, options: Request<T>) -> Response<T> {
        todo!()
    }
}

#[cfg(feature = "reqwest")]
fn test() {
    let cli = reqwest::Client::new();
}
