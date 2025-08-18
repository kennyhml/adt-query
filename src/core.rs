use crate::{
    auth::Credentials,
    common::{Cookie, CookieJar},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use http::{Response, request::Builder as RequestBuilder};
use serde::de::DeserializeOwned;
use std::{borrow::Cow, sync::Arc};
use tokio::sync::Mutex;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientNumber(pub u32);

impl Into<ClientNumber> for u32 {
    fn into(self) -> ClientNumber {
        ClientNumber(self)
    }
}

/// Contains the information of a SAP System required to connect to the ADT Services.
#[derive(Builder, Debug, Clone)]
pub struct System {
    /// The URL of the server, for example https://my-sap-system.com:8000
    #[builder(setter(into))]
    server_url: Url,

    /// Optional, the message server (load balancer) to use
    #[builder(default = None)]
    message_server: Option<String>,

    /// The SAP Router to use, required for connection to SAP GUI.
    #[builder(default = None)]
    sap_router: Option<String>,
}

impl System {
    pub fn server_url<'a>(&'a self) -> Cow<'a, Url> {
        Cow::Borrowed(&self.server_url)
    }

    pub fn message_server(&self) -> &Option<String> {
        &self.message_server
    }

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

// Represents a context within a session
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
    async fn dispatch<T>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<T>, String>
    where
        T: ResponseBody;

    /// The destination (sap system) of this session.
    fn destination(&self) -> &System;

    /// The client of the session
    fn client(&self) -> ClientNumber;

    /// The logon language of the session
    fn language(&self) -> &str;

    /// The destination (sap system) of this session.
    fn credentials(&self) -> &Credentials;

    /// The basic cookies of this session, (e.g session id, user context..)
    fn cookies(&self) -> Arc<Mutex<CookieJar>>;
}

/// Trait for any client that wants to support stateful requests
pub trait StatefulDispatch: Session + Contextualize + Sync + Send {}
impl<T: Session + Contextualize + Sync + Send> StatefulDispatch for T {}

/// Trait for any client that wants to support stateless requests
pub trait StatelessDispatch: Session + Sync + Send {}
impl<T: Session + Sync + Send> StatelessDispatch for T {}
