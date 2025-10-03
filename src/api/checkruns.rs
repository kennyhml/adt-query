use derive_builder::Builder;
use http::{HeaderMap, HeaderValue, header};
use std::borrow::Cow;

use crate::QueryParameters;
use crate::models::checkrun::{ObjectList, Reports};
use crate::operation::{Operation, Stateless};
use crate::response::Success;

#[derive(Builder, Debug, Clone)]
pub struct RunCheck<'a> {
    objects: ObjectList,

    #[builder(setter(into))]
    reporter: Cow<'a, str>,
}

impl<'a> Operation for RunCheck<'a> {
    type Response = Success<Reports>;
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::POST;

    fn url(&self) -> Cow<'static, str> {
        "checkruns".into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push("reporters", &self.reporter);
        params
    }

    fn headers(&self) -> Option<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.sap.adt.checkobjects+xml"),
        );

        Some(headers)
    }
}
