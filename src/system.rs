use derive_builder::Builder;
use http::Uri;

#[derive(Builder)]
pub struct System {
    server_url: Uri,
    #[builder(default = None)]
    message_server: Option<String>,
    #[builder(default = None)]
    sap_router: Option<String>,
}

#[derive(Builder)]
pub struct ConnectionConfig {
    credentials: Credentials,
    client: i32,
    #[builder(setter(into))]
    language: String,
}
