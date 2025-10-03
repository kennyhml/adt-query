use serde::Deserialize;

/// Wraps a collection of [`Workspace`]s
///
/// Typically the root element.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[readonly::make]
pub struct Service {
    #[serde(rename = "app:workspace", default)]
    pub workspaces: Vec<Workspace>,
}

/// Represents a feature of the service.
///
/// Provides the name of the feature, e.g `ABAP Test Cockpit` and associated Operations.
#[derive(Debug, Deserialize)]
#[serde(rename = "app:workspace")]
#[readonly::make]
pub struct Workspace {
    /// The name of the Workspace (Feature), e.g. `Change and Transport System`
    #[serde(rename = "atom:title")]
    pub title: String,

    /// A number of Operations associated with this feature
    #[serde(rename = "app:collection", default)]
    pub collections: Vec<Collection>,
}

/// An Operation of a feature, provides information as to how that Operation can be used.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[readonly::make]
pub struct Collection {
    /// The URL of the Operation, e.g `sap/bc/adt/oo/classes`
    #[serde(rename = "href", default)]
    pub href: Option<String>,

    /// The title of the Operation, e.g `Classes`
    #[serde(rename = "atom:title")]
    pub title: String,

    /// The MIME Types that this Operation can accept.
    #[serde(rename = "app:accept", default)]
    pub accept: Vec<String>,

    /// The type of resource this Operation deals with, e.g `messageclasses`
    #[serde(rename = "atom:category")]
    pub categories: Category,

    /// Template links for this Operation
    #[serde(rename = "adtcomp:templateLinks", default)]
    pub template_links: TemplateLinks,
}

// Represents a resource category
#[derive(Debug, Deserialize)]
#[serde(rename = "atom:category")]
pub struct Category {
    #[serde(rename = "@term")]
    term: String,
    #[serde(rename = "@scheme")]
    scheme: String,
}

// adtcomp:templateLinks (empty element in this case)
#[derive(Debug, Deserialize, Default)]
#[serde(rename = "atom:templateLinks")]
pub struct TemplateLinks {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discovery_response() {
        let plain_text = r#"<?xml version="1.0" encoding="utf-8"?>
            <app:service xmlns:app="http://www.w3.org/2007/app" xmlns:atom="http://www.w3.org/2005/Atom">
                <app:workspace>
                    <atom:title>Compatibility</atom:title>
                    <app:collection href="/sap/bc/adt/compatibility/graph">
                        <atom:title>Compatibility graph</atom:title>
                        <atom:category term="graph" scheme="http://www.sap.com/adt/categories/compatibility"/>
                        <adtcomp:templateLinks xmlns:adtcomp="http://www.sap.com/adt/compatibility"/>
                    </app:collection>
                </app:workspace>
                <app:workspace>
                    <atom:title>ADT Protected Mode</atom:title>
                </app:workspace>
                <app:workspace>
                    <atom:title>ADT Batch Resource</atom:title>
                    <app:collection href="/sap/bc/adt/communication/batch">
                        <atom:title>ADT Batch Resource</atom:title>
                        <app:accept>multipart/mixed</app:accept>
                        <atom:category term="batch" scheme="http://www.sap.com/adt/categories/system/communication/services"/>
                        <adtcomp:templateLinks xmlns:adtcomp="http://www.sap.com/adt/compatibility"/>
                    </app:collection>
                </app:workspace>
            </app:service>
            "#;
        let parsed: Service = serde_xml_rs::from_str(&plain_text).unwrap();
        assert_eq!(parsed.workspaces.len(), 3, "Did not parse 3 workspaces");
        assert_eq!(
            parsed.workspaces[0].title, "Compatibility",
            "Workspace title is incorrect"
        );
    }
}
