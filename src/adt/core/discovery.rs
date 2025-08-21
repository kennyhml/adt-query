use crate::endpoint::{Endpoint, Stateful, Stateless};
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

// Root element: app:service
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    #[serde(rename = "app:workspace", default)]
    workspaces: Vec<Workspace>,
}

// app:workspace
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    #[serde(rename = "atom:title")]
    title: String,
    #[serde(rename = "app:collection", default)]
    collections: Vec<Collection>,
}

// app:collection
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    #[serde(rename = "href", default)]
    href: Option<String>,
    #[serde(rename = "atom:title")]
    title: String,
    #[serde(rename = "app:accept", default)]
    accept: Option<String>,
    #[serde(rename = "atom:category", default)]
    categories: Vec<Category>,
    #[serde(rename = "adtcomp:templateLinks", default)]
    template_links: Option<TemplateLinks>,
}

// atom:category
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(rename = "@term")]
    term: String,
    #[serde(rename = "@scheme")]
    scheme: String,
}

// adtcomp:templateLinks (empty element in this case)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateLinks {}

pub struct CoreDiscovery {}

impl Endpoint for CoreDiscovery {
    type Kind = Stateless;
    type ResponseBody = Service;
    type RequestBody = ();

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}

pub struct CoreDiscoveryStateful {}

impl Endpoint for CoreDiscoveryStateful {
    type Kind = Stateful;
    type ResponseBody = Service;
    type RequestBody = ();

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_discovery_response() {
        let xml = r#"
<?xml version="1.0" encoding="utf-8"?>
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
        let xml = xml.replace("\n", "");
        let parsed: Result<Service, serde_xml_rs::Error> = serde_xml_rs::from_str(&xml);
        assert!(parsed.is_ok())
    }
}
