use crate::error::DispatchError;
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use derive_builder::Builder;
use http::{
    HeaderValue, Response,
    header::{GetAll, InvalidHeaderValue, ToStrError},
    request::Builder as RequestBuilder,
};
use std::{borrow::Cow, slice::Iter};
use thiserror::Error;
use url::Url;

#[async_trait]
pub trait RequestDispatch: Send + Sync {
    async fn dispatch_request(
        &self,
        request: RequestBuilder,
        body: Vec<u8>,
    ) -> Result<Response<Vec<u8>>, DispatchError>;
}

/// Contains the fundamental, client independent data of a SAP System.
#[derive(Builder, Debug, Clone)]
pub struct System {
    /// The name of the System, e.g. 'A4H'. Used only for organizational purposes.
    #[builder(setter(into))]
    name: String,

    /// The URL under which the system can be reached, e.g. https://my-sap-system.com:8000
    #[builder(setter(into))]
    server_url: Url,

    /// The message server to use, essentially a load-balancer.
    #[builder(default = None)]
    message_server: Option<String>,

    /// The SAP Router to use, required for connection to SAP GUI, essentially a proxy.
    ///
    /// See [Sap Router FAQ] for more information.
    ///
    /// [Sap Router FAQ]: https://community.sap.com/t5/technology-blog-posts-by-sap/sap-router-faq-s/ba-p/13372319
    #[builder(default = None)]
    sap_router: Option<String>,
}

impl System {
    /// The name of this System
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The URL under which this system can be reached.
    pub fn server_url(&self) -> Cow<'_, Url> {
        Cow::Borrowed(&self.server_url)
    }

    /// The message server of this system.
    pub fn message_server(&self) -> Option<&String> {
        self.message_server.as_ref()
    }

    /// The SAP Router of this system.
    pub fn sap_router(&self) -> Option<&String> {
        self.sap_router.as_ref()
    }
}

/// A unique identifier for a context within a session.
///
/// Context IDs are assigned incrementally, starting from 0, and are unique per session.
/// This identifier has no meaning for the server, its purely a means of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextId(pub(crate) u32);

/// Represents a user context within a session.
///
/// These are 'transactions' that hold a work process alive for their duration.
///
/// Used to avoid an expensive reload of data on the server across requests.
///
/// They are also required to modify objects as they need to be locked first.
#[derive(Debug, Clone)]
pub struct Context {
    // ID of the context, serves as internal handle to the context.
    id: ContextId,

    // When was this context created? Not related to its first usage.
    created: DateTime<Utc>,

    // The cookie that represents this context in the request
    cookie: Cookie,

    // How many requests have been made in the scope of this context
    requests_made: i32,
}

impl Context {
    pub(crate) fn new(id: ContextId, cookie: Cookie) -> Self {
        Self {
            id,
            cookie,
            created: Utc::now(),
            requests_made: 0,
        }
    }

    pub fn cookie(&self) -> &Cookie {
        &self.cookie
    }

    pub fn update(&mut self, cookie: Cookie) {
        self.cookie = cookie;
    }
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

    pub fn iter(&self) -> Iter<'_, Cookie> {
        self.cookies.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.cookies.is_empty()
    }

    pub fn clear(&mut self) {
        self.cookies.clear();
    }

    pub fn find(&self, pattern: &str) -> Option<&Cookie> {
        self.cookies.iter().find(|c| c.name.contains(pattern))
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
            self.take(&cookie.name);
            return;
        }

        if let Some(prev) = self.cookies.iter_mut().find(|v| v.name == cookie.name) {
            *prev = cookie;
        } else {
            self.cookies.push(cookie);
        }
    }

    pub fn take(&mut self, cookie: &str) -> Option<Cookie> {
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

#[derive(Debug, Default, Clone)]
pub struct QueryParameters<'a> {
    pairs: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

impl<'a> QueryParameters<'a> {
    pub fn push<'b, K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<Cow<'a, str>>,
        V: ParamValue<'b>,
        'b: 'a,
    {
        self.pairs.push((key.into(), value.as_str()));
        self
    }

    pub fn push_opt<'b, K, V>(&mut self, key: K, value: Option<V>) -> &mut Self
    where
        K: Into<Cow<'a, str>>,
        V: ParamValue<'b>,
        'b: 'a,
    {
        if let Some(value) = value {
            self.pairs.push((key.into(), value.as_str()));
        }
        self
    }

    pub fn extend<'b, I, K, V>(&mut self, iter: I) -> &mut Self
    where
        I: Iterator<Item = (K, V)>,
        K: Into<Cow<'a, str>>,
        V: ParamValue<'b>,
        'b: 'a,
    {
        self.pairs
            .extend(iter.map(|(key, value)| (key.into(), value.as_str())));
        self
    }

    pub fn add_to_url(&self, url: &mut Url) {
        let mut pairs = url.query_pairs_mut();
        pairs.extend_pairs(self.pairs.iter());
    }
}

/// A trait representing a parameter value.
pub trait ParamValue<'a> {
    fn as_str(&self) -> Cow<'a, str>;
}

impl ParamValue<'static> for bool {
    fn as_str(&self) -> Cow<'static, str> {
        if *self { "true".into() } else { "false".into() }
    }
}

impl<'a> ParamValue<'a> for &'a str {
    fn as_str(&self) -> Cow<'a, str> {
        (*self).into()
    }
}

impl ParamValue<'static> for String {
    fn as_str(&self) -> Cow<'static, str> {
        self.clone().into()
    }
}

impl<'a> ParamValue<'a> for &'a String {
    fn as_str(&self) -> Cow<'a, str> {
        (*self).into()
    }
}

impl<'a> ParamValue<'a> for Cow<'a, str> {
    fn as_str(&self) -> Cow<'a, str> {
        self.clone()
    }
}

impl<'a, 'b: 'a> ParamValue<'a> for &'b Cow<'a, str> {
    fn as_str(&self) -> Cow<'a, str> {
        (*self).clone()
    }
}

impl ParamValue<'static> for u64 {
    fn as_str(&self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

impl ParamValue<'static> for f64 {
    fn as_str(&self) -> Cow<'static, str> {
        self.to_string().into()
    }
}

impl ParamValue<'static> for DateTime<Utc> {
    fn as_str(&self) -> Cow<'static, str> {
        self.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            .into()
    }
}
