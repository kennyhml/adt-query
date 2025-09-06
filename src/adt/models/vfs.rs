/// Virtual Filesystem Models (Virtual Folders, etc..)
use derive_builder::Builder;
use serde::Serialize;
use std::borrow::Cow;

/// Preselections represent object search filters, for example:
/// ```xml
/// <vfs:preselection facet="owner">
///     <vfs:value>DEVELOPER</vfs:value>
/// </vfs:preselection>
/// ```
/// Represents a filter for the facet `owner`  with the value `DEVELOPER`.
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename = "vfs:preselection")]
pub struct Preselection<'a> {
    #[serde(rename = "@facet")]
    facet: Cow<'a, str>,

    #[serde(rename = "vfs:value")]
    value: Cow<'a, str>,
}

impl<'a, T> Into<Preselection<'a>> for (T, T)
where
    T: Into<Cow<'a, str>>,
{
    fn into(self) -> Preselection<'a> {
        Preselection {
            facet: self.0.into(),
            value: self.1.into(),
        }
    }
}

#[derive(Debug, Serialize, Builder, Clone, Default)]
#[serde(rename = "vfs:facetorder")]
pub struct FacetOrder<'a> {
    #[serde(rename = "vfs:facet")]
    #[builder(setter(each(name = "push", into)))]
    facets: Vec<Cow<'a, str>>,
}

#[derive(Debug, Serialize, Builder)]
#[serde(rename = "vfs:virtualFoldersRequest")]
#[builder(setter(strip_option))]
pub struct VirtualFoldersRequest<'a> {
    #[serde(rename = "@objectSearchPattern")]
    #[builder(setter(into), default = Cow::Borrowed("*"))]
    search_pattern: Cow<'a, str>,

    #[serde(rename = "vfs:preselection")]
    #[builder(setter(each(name = "preselection", into)), default)]
    preselections: Vec<Preselection<'a>>,

    #[serde(rename = "vfs:facetorder")]
    #[builder(default)]
    order: FacetOrder<'a>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn serialize_preselection_filter() {
        let preselection = Preselection {
            facet: "owner".into(),
            value: "DEVELOPER".into(),
        };

        let result = serde_xml_rs::to_string(&preselection).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:preselection facet="owner"><vfs:value>DEVELOPER</vfs:value></vfs:preselection>"#
        )
    }

    #[test]
    fn serialize_facet_order() {
        let order = FacetOrderBuilder::default()
            .push("owner")
            .push("package")
            .push("group")
            .push("type")
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&order).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:facetorder><vfs:facet>owner</vfs:facet><vfs:facet>package</vfs:facet><vfs:facet>group</vfs:facet><vfs:facet>type</vfs:facet></vfs:facetorder>"#
        )
    }

    #[test]
    fn serialize_virtual_folders_request() {
        let request = VirtualFoldersRequestBuilder::default()
            .preselection(("owner", "DEVELOPER"))
            .preselection(("package", "$TMP"))
            .order(
                FacetOrderBuilder::default()
                    .push("owner")
                    .push("package")
                    .push("group")
                    .push("type")
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&request).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:virtualFoldersRequest objectSearchPattern="*"><vfs:preselection facet="owner"><vfs:value>DEVELOPER</vfs:value></vfs:preselection><vfs:preselection facet="package"><vfs:value>$TMP</vfs:value></vfs:preselection><vfs:facetorder><vfs:facet>owner</vfs:facet><vfs:facet>package</vfs:facet><vfs:facet>group</vfs:facet><vfs:facet>type</vfs:facet></vfs:facetorder></vfs:virtualFoldersRequest>"#
        )
    }
}
