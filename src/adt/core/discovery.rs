use crate::endpoint::{Endpoint, Stateless};
use std::borrow::Cow;

use serde::{Deserialize, Serialize};

// Root element: app:service
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Service {
    #[serde(rename = "app:workspace", default)]
    workspaces: Vec<Workspace>,
}

// app:workspace
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Workspace {
    #[serde(rename = "atom:title")]
    title: String,
    #[serde(rename = "app:collection", default)]
    collections: Vec<Collection>,
}

// app:collection
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
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
struct Category {
    #[serde(rename = "@term")]
    term: String,
    #[serde(rename = "@scheme")]
    scheme: String,
}

// adtcomp:templateLinks (empty element in this case)
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TemplateLinks {}

struct CoreDiscovery {}

impl Endpoint for CoreDiscovery {
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

    type ResponseBody = Service;

    fn url(&self) -> Cow<'static, str> {
        "sap/bc/adt/core/discovery".into()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use url::Url;

    use super::*;
    use crate::{
        SystemBuilder, auth::Credentials, client::ClientBuilder, endpoint::StatelessQuery,
    };

    #[tokio::test]
    async fn test_discovery_endpoint() {
        let endpoint = CoreDiscovery {};
        let system = SystemBuilder::default()
            .server_url(Url::from_str("http://localhost:50000").unwrap())
            .build()
            .unwrap();

        let session = ClientBuilder::default()
            .system(system)
            .language("en")
            .client(001)
            .credentials(Credentials::new("DEVELOPER", "ABAPtr2022#01"))
            .build()
            .unwrap();

        let _response = endpoint.query(&session).await;
        todo!()
    }

    #[test]
    #[ignore]
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
