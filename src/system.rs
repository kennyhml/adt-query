use derive_builder::Builder;
use url::Url;

/// Contains the information of a SAP System required to connect to the ADT Services.
#[derive(Builder, Debug)]
pub struct ConnectionConfiguration {
    /// The URL of the server, e.g https://my-sap-system.com:8000
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
}

impl ConnectionConfiguration {
    pub fn server_url(&self) -> &Url {
        &self.server_url
    }
}
