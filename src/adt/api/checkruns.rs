use derive_builder::Builder;
use std::borrow::Cow;

use crate::QueryParameters;
use crate::adt::models::checkrun;
use crate::api::{Endpoint, Stateless, Success};

#[derive(Builder, Debug, Clone)]
pub struct RunCheck<'a> {
    objects: checkrun::ObjectList,

    #[builder(setter(into))]
    reporter: Cow<'a, str>,
}

impl Endpoint for RunCheck<'_> {
    type RequestBody = checkrun::ObjectList;
    type Response = Success<checkrun::Reports>;
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::POST;
    const CONTENT_TYPE: Option<&'static str> = Some("application/vnd.sap.adt.checkobjects+xml");

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/checkruns".into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push("reporters", &self.reporter);
        params
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

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/checkruns/reporters".into()
    }
}
