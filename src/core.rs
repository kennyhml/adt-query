use crate::{common::Cookie, system::ConnectionConfiguration};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use http::{Response, request::Builder as RequestBuilder};
use serde::de::DeserializeOwned;

/// A unique identifier for a context within a session.
///
/// Context IDs are assigned incrementally, starting from 0, and are unique per session.
/// This identifier has no meaning for the server, its purely a means of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextId(pub(crate) u32);

pub trait ResponseBody: DeserializeOwned + Send {}
impl<T: DeserializeOwned + Send> ResponseBody for T {}

pub trait Contextualize {
    /// Allocates for a new Context, this does not create any internal representation
    /// of the actual context and instead just reserves the unique id.
    fn new_context(&mut self) -> ContextId;

    /// Returns a context for the given ID, if no context. Returns None if the Context
    /// is allocated but not created or does not exist at all.
    fn context(&self, id: ContextId) -> Option<&Context>;

    /// Drops the context at the given ID and returns the ownership of it
    fn drop_context(&mut self, id: ContextId) -> Option<Context>;
}

// Represents a context within a session
#[derive(Debug)]
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
pub trait RequestDispatch {
    async fn dispatch<T>(
        &self,
        request: RequestBuilder,
        body: Option<Vec<u8>>,
    ) -> Result<Response<T>, String>
    where
        T: ResponseBody;

    fn connection(&self) -> &ConnectionConfiguration;
}

/// Trait for any client that wants to support stateful requests
pub trait StatefulDispatch: RequestDispatch + Contextualize + Sync + Send {}

/// Trait for any client that wants to support stateless requests
pub trait StatelessDispatch: RequestDispatch + Sync + Send {}
