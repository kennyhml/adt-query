use std::borrow::Cow;

use derive_builder::Builder;
use url::Url;

use crate::auth::Credentials;

/// Contains the information of a SAP System required to connect to the ADT Services.
#[derive(Builder, Debug)]
pub struct ConnectionConfiguration {
    /// The URL of the server, for example https://my-sap-system.com:8000
    #[builder(setter(into))]
    server_url: Url,

    /// Optional, the message server (load balancer) to use
    #[builder(default = None)]
    message_server: Option<String>,

    /// The SAP Router to use, required for connection to SAP GUI.
    #[builder(default = None)]
    sap_router: Option<String>,

    // The client to connect on
    client: i32,

    // The language to connect with, e.g 'EN', 'DE'..
    #[builder(setter(into))]
    language: String,

    credentials: Credentials,
}

impl ConnectionConfiguration {
    pub fn server_url<'a>(&'a self) -> Cow<'a, Url> {
        Cow::Borrowed(&self.server_url)
    }

    pub fn message_server(&self) -> &Option<String> {
        &self.message_server
    }

    pub fn sap_router(&self) -> &Option<String> {
        &self.sap_router
    }

    pub fn client(&self) -> i32 {
        self.client
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }
}
