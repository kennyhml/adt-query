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

#[derive(Debug, Copy)]
pub enum ADTHeader {
    // Enable request runtime profiling
    Profiling(bool),
    // The current time on the server for clock synchronization
    ServerTime(String),
    // The server instance that processed the request
    ServerInstance(String),
    // The internal server instance that processed the request
    InternalServerInstance(String),
    // Whether the session is in a softstate
    Softstate(bool),
    // The HTTP Version that is used, typically HTTP 1.1
    HtppVersion(http::Version),
}

impl ADTHeader {
    pub const PROFILING: &'static str = "X-sap-adt-profiling";
    pub const SERVER_TIME: &'static str = "X-sap-adt-profiling";
    pub const SERVER_TIME: &'static str = "X-sap-adt-profiling";
}
