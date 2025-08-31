use crate::adt::models::discovery;
use crate::api::{Endpoint, Stateful, Stateless, Success};
use std::borrow::Cow;

pub struct CoreDiscovery {}

impl Endpoint for CoreDiscovery {
    type Kind = Stateless;
    type Response = Success<discovery::Service>;
    type RequestBody = ();

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}

pub struct CoreDiscoveryStateful {}

impl Endpoint for CoreDiscoveryStateful {
    type Kind = Stateful;
    type Response = Success<discovery::Service>;
    type RequestBody = ();

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}
