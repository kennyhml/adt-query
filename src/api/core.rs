use crate::models::discovery;
use crate::operation::{Operation, Stateless};
use crate::response::Success;
use std::borrow::Cow;

pub struct CoreDiscovery {}

impl Operation for CoreDiscovery {
    type Kind = Stateless;

    type Response = Success<discovery::Service>;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "core/discovery".into()
    }
}
