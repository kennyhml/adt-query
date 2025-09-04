use std::borrow::Cow;

use derive_builder::Builder;
use http::{HeaderMap, HeaderValue};

use crate::{
    adt::models::adtcore,
    adt::models::program::AbapProgram,
    api::{Accept, CacheControlled, Endpoint, Plain, Stateless},
};

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Program {
    /// The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: String,

    /// The version of the program to get the data of, e.g. `inactive` or `workingArea` or `active`
    /// If not specified, the inactive version is returned unless only an active version exists.
    #[builder(default=None)]
    version: Option<adtcore::Version>,

    #[builder(setter(into), default=None)]
    etag: Option<String>,
}

impl Endpoint for Program {
    type Response = CacheControlled<AbapProgram>;
    type RequestBody = ();

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;
    const ACCEPT: Accept = Some("application/vnd.sap.adt.programs.programs.v3+xml");

    fn url(&self) -> Cow<'static, str> {
        let mut url = Cow::Owned(format!("sap/bc/adt/programs/programs/{}", self.name));
        if let Some(version) = &self.version {
            url = Cow::Owned(format!("{}?version={}", url, version.as_str()));
        }
        url
    }

    /// Headers need to handle whether we have a cached version locally and provide the ETag.
    fn headers(&self) -> Option<http::HeaderMap> {
        let mut map = HeaderMap::new();
        match &self.etag {
            None => map.insert("Cache-Control", HeaderValue::from_static("no-cache")),
            Some(etag) => map.insert("If-None-Match", HeaderValue::from_str(etag).unwrap()),
        };
        Some(map)
    }
}

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProgramSource {
    // The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: String,

    // The version of the program to get the data of, e.g. `inactive`
    #[builder(default=None)]
    version: Option<adtcore::Version>,

    #[builder(setter(into), default=None)]
    etag: Option<String>,
}

impl Endpoint for ProgramSource {
    type RequestBody = ();
    type Response = CacheControlled<Plain<String>>;

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;
    const ACCEPT: Accept = Some("text/plain");

    fn url(&self) -> Cow<'static, str> {
        let mut url = Cow::Owned(format!(
            "sap/bc/adt/programs/programs/{}/source/main",
            self.name
        ));
        if let Some(version) = &self.version {
            url = Cow::Owned(format!("{}?version={}", url, version.as_str()));
        }
        url
    }

    /// Headers need to handle whether we have a cached version locally and provide the ETag.
    fn headers(&self) -> Option<http::HeaderMap> {
        let mut map = HeaderMap::new();
        match &self.etag {
            None => map.insert("Cache-Control", HeaderValue::from_static("no-cache")),
            Some(etag) => map.insert("If-None-Match", HeaderValue::from_str(etag).unwrap()),
        };
        Some(map)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn can_create_program_without_etag() {
        ProgramBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .build()
            .unwrap();
    }

    #[test]
    fn can_create_program_with_etag() {
        ProgramBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build()
            .unwrap();
    }

    #[test]
    fn program_name_is_mandatory() {
        let result = ProgramBuilder::default()
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build();

        assert!(matches!(result, Err(_)), "Name should not be optional");
    }
}
