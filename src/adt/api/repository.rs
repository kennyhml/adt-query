use std::borrow::Cow;

use derive_builder::Builder;

use crate::{
    QueryParameters,
    adt::models::{
        facets::Facets,
        serialize::IntoXmlRoot,
        vfs::{FacetOrder, Preselection, VirtualFoldersRequest, VirtualFoldersResult},
    },
    api::{Endpoint, Stateless, Success},
    error::SerializeError,
};

#[derive(Debug, Clone)]
pub enum Operation {
    Expand,
    Count,
}

impl Operation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Expand => "expand",
            Self::Count => "count",
        }
    }
}

/// Fetches contents from the repository information system as virtual folders
///
/// Responsible ABAP REST Handler: `CL_RIS_ADT_RES_VIRTUAL_FOLDERS`
///
/// It is only possible to get one layer of subfolders / objects with per call,
/// we cannot exploring the system recursively.
#[derive(Debug, Builder)]
#[builder(setter(strip_option))]
pub struct RepositoryContent<'a> {
    /// The search pattern that the object names are filtered by in the object selection.
    #[builder(default = Cow::Borrowed("*"))]
    search_pattern: Cow<'a, str>,

    /// Defines how the relevant objects should be selected, see [`Preselection`]
    preselections: Vec<Preselection<'a>>,

    /// The desired facets. If left empty, a list of [`Object`] for the preselection is returned.
    ///
    /// **Note:** Despite being a list of items, as per the servers behavior, only the first
    /// facet in the list is actually ever used.
    #[builder(default)]
    order: FacetOrder<'a>,

    /// Either `expand`, which returns the desired objects or `count`, which returns the number of matches.
    ///
    /// When unspecified in the query, the default behavior is `expand`.
    #[builder(default)]
    operation: Option<Operation>,

    /// Whether the descriptions of the objects should be included in the result.
    ///
    /// When unspecified in the query, the default behavior is `False`.
    #[builder(default)]
    ignore_short_descriptions: Option<bool>,

    /// Whether a version preselection should be taken into consideration. Must be set
    /// for the value in the preselection to be used.
    ///
    /// When unspecified in the query, the default behavior is `False`.
    ///
    /// **Negatively impacts the performance (+100ms), use only if needed.**
    #[builder(default)]
    with_versions: Option<bool>,
}

impl<'a> Endpoint for RepositoryContent<'a> {
    type Kind = Stateless;

    type Response = Success<VirtualFoldersResult<'a>>;

    const METHOD: http::Method = http::Method::POST;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/repository/informationsystem/virtualfolders/contents".into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push_opt("ignoreShortDescriptions", self.ignore_short_descriptions);
        params.push_opt("withVersions", self.with_versions);
        params.push_opt("operation", self.operation.as_ref().map(|v| v.as_str()));
        params
    }

    fn body(&self) -> Option<Result<String, SerializeError>> {
        let body =
            VirtualFoldersRequest::new(&self.search_pattern, &self.preselections, &self.order);

        Some(body.into_xml_root())
    }
}

#[derive(Debug, Default)]
pub struct AvailableFacets {}

impl Endpoint for AvailableFacets {
    type Kind = Stateless;

    type Response = Success<Facets>;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "/sap/bc/adt/repository/informationsystem/virtualfolders/facets".into()
    }
}
