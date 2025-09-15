use std::borrow::Cow;

use derive_builder::Builder;
use http::{HeaderMap, HeaderValue, header};

use crate::endpoint::{Endpoint, Stateless};
use crate::response::{CacheControlled, Plain, Success};
use crate::{
    QueryParameters,
    adt::models::{
        abapsource::ObjectStructureElement, adtcore, atom::VersionFeed, program::AbapProgram,
    },
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

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

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
            None => map.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            Some(etag) => map.insert(header::IF_NONE_MATCH, HeaderValue::from_str(etag).unwrap()),
        };
        map.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/vnd.sap.adt.programs.programs.v3+xml"),
        );
        Some(map)
    }
}

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProgramSource<'a> {
    // The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: Cow<'a, str>,

    // The version of the program to get the data of, e.g. `inactive`
    #[builder(default)]
    version: Option<adtcore::Version>,

    #[builder(setter(into), default)]
    etag: Option<Cow<'a, str>>,
}

impl<'a> Endpoint for ProgramSource<'a> {
    type Response = CacheControlled<Plain<'a>>;

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

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
            None => map.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            Some(etag) => map.insert(header::IF_NONE_MATCH, HeaderValue::from_str(etag).unwrap()),
        };
        map.insert(header::ACCEPT, HeaderValue::from_static("text/plain"));
        Some(map)
    }
}

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProgramVersions<'a> {
    /// The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: Cow<'a, str>,
}

impl Endpoint for ProgramVersions<'_> {
    type Response = Success<VersionFeed>;

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        format!(
            "sap/bc/adt/programs/programs/{}/source/main/versions",
            self.name
        )
        .into()
    }

    fn headers(&self) -> Option<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/atom+xml;type=feed"),
        );
        Some(headers)
    }
}

#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct ProgramStructure<'a> {
    /// The name of the program, for example `zwegwerf1`
    #[builder(setter(into))]
    name: Cow<'a, str>,

    /// The version of the program to get the data of, see [`adtcore::Version`]
    /// If not specified in the query, the inactive version is the default if one exists.
    #[builder(default=None)]
    version: Option<adtcore::Version>,

    /// Retrieve short descriptions
    #[builder(setter(into))]
    short_descriptions: Option<bool>,
}

impl Endpoint for ProgramStructure<'_> {
    type Response = Success<ObjectStructureElement>;

    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        format!(
            "sap/bc/adt/programs/programs/{}/source/main/versions",
            self.name
        )
        .into()
    }

    fn headers(&self) -> Option<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static("application/atom+xml;type=feed"),
        );
        Some(headers)
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
