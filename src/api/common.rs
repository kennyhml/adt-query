use chrono::{DateTime, NaiveDateTime, Utc};
use http::{
    HeaderName, HeaderValue,
    header::{InvalidHeaderValue, ToStrError},
};
use thiserror::Error;

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
pub struct Cookie {
    name: String,
    value: String,
    path: String,

    include: bool,

    expires: Option<DateTime<Utc>>,
    domain: Option<String>,
}

#[derive(Error, Debug)]
pub enum CookieError {
    #[error("Could not parse Cookie: '{0}'")]
    ParseError(String),

    #[error("Could not parse Cookie Date: '{0}'")]
    DateParseError(#[from] chrono::ParseError),

    #[error("Could not parse Cookie Header: {0}")]
    HeaderError(#[from] ToStrError),
}

impl Cookie {
    pub const SSO2: &'static str = "MYSAPSSO2";
    pub const SAP_SESSIONID: &'static str = "SAP_SESSIONID_";
    pub const USER_CONTEXT: &'static str = "sap-usercontext";
    pub const SAP_CONTEXT_ID: &'static str = "sap-contextid";

    pub fn parse_from_header(header: &HeaderValue) -> Result<Self, CookieError> {
        Self::parse(header.to_str()?)
    }

    pub fn parse(cookie: &str) -> Result<Self, CookieError> {
        let (name, data) = cookie
            .split_once("=")
            .ok_or(CookieError::ParseError(cookie.to_owned()))?;

        let mut value_iterator = data.split("; ");
        let value = value_iterator
            .next()
            .ok_or(CookieError::ParseError(cookie.to_owned()))?;

        let mut result = Self {
            name: name.to_owned(),
            value: value.to_owned(),
            include: true,
            expires: None,
            path: String::new(),
            domain: None,
        };

        while let Some(pair) = value_iterator.next() {
            let (name, value) = pair
                .split_once("=")
                .ok_or(CookieError::ParseError(pair.to_owned()))?;

            match name {
                "expires" => {
                    result.expires = Some(
                        NaiveDateTime::parse_from_str(value, "%a, %d-%b-%Y %H:%M:%S %Z")?.and_utc(),
                    );
                }
                "path" => result.path = value.to_owned(),
                "domain" => result.domain = Some(value.to_owned()),
                _ => continue,
            }
        }
        Ok(result)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn domain(&self) -> &Option<String> {
        &self.domain
    }

    pub fn as_cookie_pair(&self) -> String {
        format!("{}={}", self.name, self.value)
    }
}

#[derive(Debug)]
pub struct CookieJar {
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn set_cookie_from_header(&mut self, header: &HeaderValue) {
        let (name, value) = header.to_str().unwrap().split_once("=").unwrap();

        let parsed = self.get_parsed_cookies();

        todo!()
    }

    pub fn get_parsed_cookies(&self) -> Vec<Cookie> {
        todo!()
    }

    pub fn to_header(&self) -> HeaderValue {
        todo!()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sso_cookie() {
        let name = "MYSAPSSO2";
        let value = "AjQxMDMBAe2qeadadwadwadwa";
        let path = "/";
        let domain = "localhost";
        let expires = "Tue, 01-Jan-1980 00:00:01 GMT";

        let cookie = Cookie::parse(&format!(
            "{name}={value}; path={path}; domain={domain}; expires={expires}"
        ));
        println!("{cookie:?}");
        assert!(!cookie.is_err(), "Parsing SSO2 Cookie failed.");

        let cookie = cookie.unwrap();
        assert_eq!(cookie.value, value);
        assert_eq!(cookie.name, name);
        assert_eq!(cookie.path, path);
        assert_eq!(cookie.domain, Some(domain.to_owned()));
        assert_ne!(cookie.expires, None);
    }
}
