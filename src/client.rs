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
    async fn login() -> Client<T, Connected> {
        todo!()
    }
}

impl<T: HTTPClient> Client<T, Connected> {
    async fn logout() -> Client<T, ()> {
        todo!()
    }
}

pub struct SAPServer {
    server_url: String,
    message_server: Option<String>,
    sap_router: Option<String>,

    credentials: Credentials,
    client: i32,
    language: String,
}

#[cfg(feature = "reqwest")]
fn test() {
    let cli = reqwest::Client::new();
}
