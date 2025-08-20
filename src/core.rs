use crate::{
    auth::Credentials,
    common::{Cookie, CookieJar},
    error::QueryError,
};
use arc_swap::ArcSwap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::{Response, request::Builder as RequestBuilder};
use serde::{Serialize, de::DeserializeOwned};
use std::{borrow::Cow, sync::Arc};
use url::Url;

/// Wraps a client number to connect to a SAP System with.
///
/// See [What is SAP Client?](https://erpiseasy.com/2022/10/09/what-is-sap-client/)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientNumber(pub u32);

impl Into<ClientNumber> for u32 {
    fn into(self) -> ClientNumber {
        ClientNumber(self)
    }
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
    pub fn server_url<'a>(&'a self) -> Cow<'a, Url> {
        Cow::Borrowed(&self.server_url)
    }

    /// The message server of this system.
    pub fn message_server(&self) -> &Option<String> {
        &self.message_server
    }

    /// The SAP Router of this system.
    pub fn sap_router(&self) -> &Option<String> {
        &self.sap_router
    }
}

/// A unique identifier for a context within a session.
///
/// Context IDs are assigned incrementally, starting from 0, and are unique per session.
/// This identifier has no meaning for the server, its purely a means of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextId(pub(crate) u32);

pub trait ResponseBody: DeserializeOwned + Send {}
impl<T: DeserializeOwned + Send> ResponseBody for T {}

pub trait RequestBody: Serialize + Send {}
impl<T: Serialize + Send> RequestBody for T {}

pub trait Contextualize {
    /// Allocates for a new Context, this should  not create any internal representation
    /// of the actual context and instead just reserves the unique id.
    fn new_context(&mut self) -> ContextId;

    /// Returns a context for the given ID. Returns None if the Context
    /// is allocated but not created or does not exist at all.
    fn context(&self, id: ContextId) -> Option<&Context>;

    /// Drops the context at the given ID and returns the ownership of it
    fn drop_context(&mut self, id: ContextId) -> Option<Context>;
}

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
    _id: ContextId,

    // When was this context created? Not related to its first usage.
    created: DateTime<Utc>,

    // The cookie that represents this context in the request
    cookie: Cookie,

    // How many requests have been made in the scope of this context
    requests_made: i32,
}

/// Trait that handles actually dispatching a request, this isnt concerned with whether the request
/// is stateful or stateless or whatever as that is handled by the query traits. This trait is only
/// concerned with actually dispatching a request to the system.
#[async_trait]
pub trait Session {
    async fn dispatch(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<Vec<u8>>, QueryError>;

    /// The destination (sap system) of this session.
    fn destination(&self) -> &System;

    /// The client of the session
    fn client(&self) -> ClientNumber;

    /// The logon language of the session
    fn language(&self) -> &str;

    /// The destination (sap system) of this session.
    fn credentials(&self) -> &Credentials;

    /// The basic cookies of this session, (e.g session id, user context..)
    fn cookies(&self) -> &ArcSwap<CookieJar>;

    /// Drops all the cookies to essentially drop the session.
    ///
    /// **Caution:** This does not automatically drop the session and contexts.
    fn drop_session(&mut self) {
        self.cookies().store(Arc::new(CookieJar::new()));
    }

    /// Checks whether the client is logged on based on the session id cookie.
    ///
    /// **Caution:** This does not guarantee the session has not timed out or is invalid.
    fn is_logged_on(&self) -> bool {
        self.cookies().load().find(Cookie::SAP_SESSIONID).is_some()
    }

    /// Returns a representation of the current session as `{dst}: (client, language)`
    fn info(&self) -> String {
        format!(
            "{} ({}, '{}')",
            self.destination().name(),
            self.client().0,
            self.language()
        )
    }
}

/// Trait for any client that wants to support stateful requests
pub trait StatefulDispatch: Session + Contextualize + Sync + Send {}
impl<T: Session + Contextualize + Sync + Send> StatefulDispatch for T {}

/// Trait for any client that wants to support stateless requests
pub trait StatelessDispatch: Session + Sync + Send {}
impl<T: Session + Sync + Send> StatelessDispatch for T {}
