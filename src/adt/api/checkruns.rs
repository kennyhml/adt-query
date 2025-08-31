use derive_builder::Builder;
use std::borrow::Cow;

use crate::adt::models::checkrun;
use crate::api::{Endpoint, Stateless, Success};

#[derive(Builder, Debug, Clone)]
pub struct RunCheck {
    objects: checkrun::ObjectList,

    #[builder(setter(into))]
    reporter: String,
}

impl Endpoint for RunCheck {
    type RequestBody = checkrun::ObjectList;
    type Response = Success<checkrun::Reports>;
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::POST;
    const CONTENT_TYPE: Option<&'static str> = Some("application/vnd.sap.adt.checkobjects+xml");

    fn url(&self) -> Cow<'static, str> {
        Cow::Owned(format!("sap/bc/adt/checkruns?reporters={}", self.reporter))
    }

    fn body(&self) -> Option<&Self::RequestBody> {
        Some(&self.objects)
    }
}

pub struct Reporters {}

impl Endpoint for Reporters {
    type RequestBody = ();
    type Response = Success<checkrun::Reporters>;
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> std::borrow::Cow<'static, str> {
        "sap/bc/adt/checkruns/reporters".into()
    }
}
