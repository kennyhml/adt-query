use http::{HeaderName, HeaderValue, header::InvalidHeaderValue};

/// The runtime profiling options that ADT offers.
#[derive(Debug, Clone)]
pub struct RuntimeProfilingKind(pub &'static str);

impl RuntimeProfilingKind {
    /// Capture only the server processing time, returned in `server_time=x` (ms).
    pub const SERVER_TIME: RuntimeProfilingKind = RuntimeProfilingKind("server-time");
}

/// Custom ADT Headers (X-sap-adt...) for requests to the backend.
#[derive(Debug)]
pub struct Header;

impl Header {
    const PROFILING: &'static str = "X-sap-adt-profiling";
    const RUNTIME_TRACING: &'static str = "X-adt-runtime-tracing";
    const SERVER_INSTANCE: &'static str = "X-sap-adt-server-instance";
    const SOFTSTATE: &'static str = "X-sap-adt-softstate";
    const SESSIONTYPE: &'static str = "X-sap-adt-sessiontype";
}

/// Custom ADT Headers (X-sap-adt...) for requests to the backend.
#[derive(Debug)]
pub struct Cookie;

impl Cookie {
    pub const SSO2: &'static str = "MYSAPSSO2";
    pub const SAP_SESSIONID: &'static str = "SAP_SESSIONID_";
    pub const USER_CONTEXT: &'static str = "sap-usercontext";
    pub const SAP_CONTEXT_ID: &'static str = "sap-contextid";
}

#[derive(Debug, Clone)]
pub enum ADTHeaderValue {
    /// The profiling mode, see [`RuntimeProfilingKind`] for the possible options.
    ProfilingKind(RuntimeProfilingKind),
    /// The server instance should process the request, requires clarification.
    ServerInstance(String),
    /// Whether the session is in a [soft state](https://community.sap.com/t5/technology-blog-posts-by-sap/how-to-use-soft-state-support-for-odata-services/bc-p/13259257)
    Softstate(bool),
    /// Trace Request, requires clarification, see cl_atradt_instant_tracing->s_rest_rfc_endpoint
    RuntimeTracing(String),
    // Internal Server Instance: Used internally by ADT when redirecting the request to another instance.
}

impl ADTHeaderValue {
    pub fn name(&self) -> HeaderName {
        match self {
            ADTHeaderValue::ProfilingKind(_) => HeaderName::from_static(Header::PROFILING),
            ADTHeaderValue::ServerInstance(_) => HeaderName::from_static(Header::SERVER_INSTANCE),
            ADTHeaderValue::Softstate(_) => HeaderName::from_static(Header::SOFTSTATE),
            ADTHeaderValue::RuntimeTracing(_) => HeaderName::from_static(Header::RUNTIME_TRACING),
        }
    }

    pub fn value(&self) -> Result<HeaderValue, InvalidHeaderValue> {
        match &self {
            ADTHeaderValue::ProfilingKind(v) => v.0.try_into(),
            ADTHeaderValue::ServerInstance(v) | ADTHeaderValue::RuntimeTracing(v) => v.try_into(),
            ADTHeaderValue::Softstate(v) => Ok((*v as i32).into()),
        }
    }
}
