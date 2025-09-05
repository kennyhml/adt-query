use std::borrow::Cow;

use derive_builder::Builder;
use http::{HeaderMap, HeaderValue};

use crate::{
    QueryParameters,
    adt::models::{adtcore, program::AbapProgram},
    api::{Accept, CacheControlled, Endpoint, Plain, Stateless},
};

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct Program<'a> {
    /// The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: Cow<'a, str>,

    /// The version of the program to get the data of, see `[adtcore::Version]`
    /// If not specified in the query, the inactive version is the default if one exists.
    #[builder(default=None)]
    version: Option<adtcore::Version>,

    /// Etag of the program used for caching purposes, etags of programs are compared
    /// to determine whether any changes have been made to the program.
    #[builder(setter(into), default=None)]
    etag: Option<Cow<'a, str>>,
}

impl Endpoint for Program<'_> {
    type Response = CacheControlled<AbapProgram>;
    type RequestBody = ();

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;
    const ACCEPT: Accept = Some("application/vnd.sap.adt.programs.programs.v3+xml");

    fn url(&self) -> Cow<'static, str> {
        format!("sap/bc/adt/programs/programs/{}", self.name).into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push_opt("version", self.version.clone());
        params
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
        format!("sap/bc/adt/programs/programs/{}/source/main", self.name).into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push_opt("version", self.version.clone());
        params
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
    fn can_create_data_query_without_etag() {
        ProgramBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .build()
            .unwrap();
    }

    #[test]
    fn can_create_source_query_without_etag() {
        ProgramSourceBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .build()
            .unwrap();
    }

    #[test]
    fn can_create_data_query_with_etag() {
        ProgramBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build()
            .unwrap();
    }

    #[test]
    fn can_create_source_query_with_etag() {
        ProgramSourceBuilder::default()
            .name("ZDEMO01")
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build()
            .unwrap();
    }

    #[test]
    fn program_data_query_name_is_mandatory() {
        let result = ProgramBuilder::default()
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build();

        assert!(matches!(result, Err(_)), "Name should not be optional");
    }

    #[test]
    fn program_source_query_name_is_mandatory() {
        let result = ProgramSourceBuilder::default()
            .version(adtcore::Version::Active)
            .etag("202508101355580001")
            .build();

        assert!(matches!(result, Err(_)), "Name should not be optional");
    }
}
