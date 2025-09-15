use crate::adt::models::discovery;
use crate::endpoint::{Endpoint, Stateless};
use crate::response::Success;
use std::borrow::Cow;

pub struct CoreDiscovery {}

impl Endpoint for CoreDiscovery {
    type Kind = Stateless;

    type Response = Success<discovery::Service>;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}
