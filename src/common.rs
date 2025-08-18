use chrono::{DateTime, NaiveDateTime, Utc};
use http::{
    HeaderName, HeaderValue,
    header::{GetAll, InvalidHeaderValue, ToStrError},
};
use thiserror::Error;
use url::Url;

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

/// Represents a HTTP Cookie that can be parsed from a `Set-Cookie` Header
///
/// Represents the content of a [`CookieJar`] that is used for session handling.
///
/// See [RFC 6265 Section 5.2][rfc] for more information.
///
/// [rfc]: https://datatracker.ietf.org/doc/html/rfc6265#section-5.2
#[derive(Debug, Clone)]
pub struct Cookie {
    /// Name of the cookie, e.g `MYSAPSSO2`, `sap-contextid`, etc..
    name: String,

    /// Value of the cookie, typically just a string of data we dont particularly care about
    value: String,

    /// What paths should the cookie be included in? Could be `/` for all or e.g `sap/bc/adt`
    path: Option<String>,

    /// What domain this cookie should be included for
    domain: Option<String>,

    /// When this cookie will expire. SAP sets it to base UTC time (1st of January 1980) to indicate removal
    expires: Option<DateTime<Utc>>,
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
            expires: None,
            path: None,
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
                "path" => result.path = Some(value.replace(";", "")),
                "domain" => result.domain = Some(value.replace(";", "")),
                _ => {}
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

    pub fn path(&self) -> &Option<String> {
        &self.path
    }

    pub fn domain(&self) -> &Option<String> {
        &self.domain
    }

    pub fn as_cookie_pair(&self) -> String {
        format!("{}={}", self.name, self.value)
    }

    pub fn is_allowed_for_destination(&self, dst: &Url) -> bool {
        let path = dst.to_string();

        self.domain.as_ref().map_or(true, |d| path.contains(d))
            && self.path.as_ref().map_or(true, |p| path.contains(p))
    }

    pub fn expired(&self) -> bool {
        self.expires.map(|exp| exp < Utc::now()).unwrap_or(false)
    }
}

/// A collection of cookies and associated data, enables handling of `Set-Cookie` headers.
///
/// For each `Stateful` session, a seperate Jar should be maintained in favor of concurrency.
#[derive(Debug, Clone)]
pub struct CookieJar {
    /// The cookies that are part of this Jar, see [`Cookie`]
    cookies: Vec<Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    pub fn set_cookie_from_header(&mut self, header: &HeaderValue) {
        self.set_cookie(header.to_str().unwrap())
    }

    pub fn set_from_multiple_headers(&mut self, headers: GetAll<'_, HeaderValue>) {
        headers
            .iter()
            .for_each(|h| self.set_cookie(h.to_str().unwrap()));
    }

    pub fn set_cookie(&mut self, cookie: &str) {
        let cookie = Cookie::parse(cookie).unwrap();

        // SAP indicates that a cookie should be removed by setting it as expired.
        if cookie.expired() {
            self.drop_cookie(&cookie.name);
            return;
        }

        if let Some(prev) = self.cookies.iter_mut().find(|v| v.name == cookie.name) {
            *prev = cookie;
        } else {
            self.cookies.push(cookie);
        }
    }

    pub fn drop_cookie(&mut self, cookie: &str) -> Option<Cookie> {
        let pos = self.cookies.iter().position(|c| c.name == cookie)?;
        Some(self.cookies.remove(pos))
    }

    pub fn to_header(&self, destination: &Url) -> Result<HeaderValue, InvalidHeaderValue> {
        HeaderValue::from_str(
            &self
                .cookies
                .iter()
                .filter(|cookie| cookie.is_allowed_for_destination(&destination))
                .map(Cookie::as_cookie_pair)
                .collect::<Vec<String>>()
                .join("; "),
        )
    }
}

#[derive(Debug, Clone)]
pub enum ADTHeaderValue {
    /// The profiling mode, see [] for the possible options.
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
    use std::str::FromStr;

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
        assert!(!cookie.is_err(), "Parsing SSO2 Cookie failed.");

        let cookie = cookie.unwrap();
        assert_eq!(cookie.as_cookie_pair(), format!("{name}={value}"));
        assert_eq!(cookie.path, Some(path.to_owned()));
        assert_eq!(cookie.domain, Some(domain.to_owned()));
        assert_ne!(cookie.expires, None);
        assert_eq!(cookie.expired(), true);
    }

    #[test]
    fn test_cookie_jar() {
        let name = "MYSAPSSO2";
        let value = "AjQxMDMBAe2qeadadwadwadwa";
        let path = "/";
        let domain = "localhost";

        let destination = Url::from_str("http://localhost:50000/sap/bc/adt").unwrap();

        let mut jar = CookieJar::new();
        jar.set_cookie(&format!("{name}={value}; path={path}; domain={domain};"));

        assert_eq!(
            jar.to_header(&destination).unwrap().to_str().unwrap(),
            format!("{name}={value}")
        );

        let dropped = jar.drop_cookie(name);
        assert!(dropped.is_some(), "Cookie was not dropped.");

        assert!(
            jar.to_header(&destination)
                .unwrap()
                .to_str()
                .unwrap()
                .is_empty(),
            "Jar is not empty."
        )
    }
}
