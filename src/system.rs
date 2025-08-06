use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::Uri;

/// Contains the information of a SAP System required to connect to the ADT Services.
#[derive(Builder, Debug)]
pub struct ConnectionConfiguration {
    /// The URL of the server, e.g https://my-sap-system.com:8000
    #[builder(setter(into))]
    server_url: Uri,

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

pub struct StatefulSession {
    start: DateTime<Utc>,
    session_id: String,
}

pub struct StatelessSession {
    start: DateTime<Utc>,
    session_id: String,
}

pub enum Session {
    Stateful(StatefulSession),
    StatelessSession(StatelessSession),
}
