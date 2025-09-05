use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "atom:feed")]
#[readonly::make]
pub struct VersionFeed {
    #[serde(rename = "atom:title")]
    pub title: String,

    #[serde(rename = "atom:updated")]
    pub updated: DateTime<Utc>,

    #[serde(rename = "atom:entry")]
    pub entries: Vec<VersionEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "atom:entry")]
#[readonly::make]
pub struct VersionEntry {
    #[serde(rename = "atom:title")]
    pub description: Option<String>,

    #[serde(rename = "atom:author")]
    pub author: Author,

    #[serde(rename = "atom:content")]
    pub content: Content,

    #[serde(rename = "atom:updated")]
    pub updated: DateTime<Utc>,

    #[serde(rename = "atom:id")]
    pub id: i32,

    #[serde(rename = "atom:link", default)]
    pub transport: Vec<TransportLink>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "atom:author")]
#[readonly::make]
pub struct Author {
    #[serde(rename = "atom:name")]
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "atom:content")]
#[readonly::make]
pub struct Content {
    #[serde(rename = "@type")]
    pub kind: String,

    #[serde(rename = "@src")]
    pub source: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "atom:link")]
#[readonly::make]
pub struct Link {
    #[serde(rename = "@href")]
    pub href: String,

    #[serde(rename = "@rel")]
    pub rel: Option<String>,

    #[serde(rename = "@type")]
    pub kind: Option<String>,

    #[serde(rename = "@etag")]
    pub etag: Option<String>,

    #[serde(rename = "@title")]
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "atom:link")]
#[readonly::make]
pub struct TransportLink {
    #[serde(rename = "@adtcore:name")]
    pub transport: String,

    #[serde(rename = "@href")]
    pub href: String,

    #[serde(rename = "@rel")]
    pub rel: String,

    #[serde(rename = "@type")]
    pub kind: String,

    #[serde(rename = "@title")]
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_atom_link() {
        let content = r#"<?xml version="1.0" encoding="UTF-8"?>
                         <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="source/main/versions" rel="http://www.sap.com/adt/relations/versions"/>"#;
        let result: Link = serde_xml_rs::from_str(content).unwrap();

        assert_eq!(
            result,
            Link {
                href: String::from("source/main/versions"),
                rel: Some(String::from("http://www.sap.com/adt/relations/versions")),
                kind: None,
                etag: None,
                title: None,
            },
        )
    }

    #[test]
    fn parse_version_feed() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><atom:feed xmlns:atom="http://www.w3.org/2005/Atom" xmlns:adtcore="http://www.sap.com/adt/core">
                            <atom:title>Version List of ZWEGWERF1 (REPS)</atom:title>
                            <atom:updated>1970-01-01T10:11:23Z</atom:updated>
                            <atom:entry>
                                <atom:author>
                                <atom:name>DEVELOPER</atom:name>
                                </atom:author>
                                <atom:content type="text/plain" src="/sap/bc/adt/programs/programs/zwegwerf1/source/main/versions/19700101101123/99999/content"/>
                                <atom:id>99999</atom:id>
                                <atom:updated>2025-09-05T17:44:13Z</atom:updated>
                            </atom:entry>
                            <atom:entry>
                                <atom:author>
                                <atom:name>DEVELOPER</atom:name>
                                </atom:author>
                                <atom:content type="text/plain" src="/sap/bc/adt/programs/programs/zwegwerf1/source/main/versions/19700101101123/00000/content"/>
                                <atom:id>00000</atom:id>
                                <atom:updated>2025-08-30T21:49:44Z</atom:updated>
                            </atom:entry>
                            </atom:feed>"#;
        let _result: VersionFeed = serde_xml_rs::from_str(plain).unwrap();
    }

    #[test]
    fn deserialize_version_feed_with_transports() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><atom:feed xmlns:atom="http://www.w3.org/2005/Atom" xmlns:adtcore="http://www.sap.com/adt/core">
                            <atom:title>Version List of Z_BADI_CHECK (REPS)</atom:title>
                            <atom:updated>1970-01-01T10:11:23Z</atom:updated>
                            <atom:entry>
                                <atom:author>
                                <atom:name>ROSENKRANZ</atom:name>
                                </atom:author>
                                <atom:content type="text/plain" src="/sap/bc/adt/programs/programs/z_badi_check/source/main/versions/19700101101123/00000/content"/>
                                <atom:id>00000</atom:id>
                                <atom:link adtcore:name="QE1K900019" href="/sap/bc/adt/vit/wb/object_type/%20%20%20%20rq/object_name/QE1K900019" rel="http://www.sap.com/adt/relations/transport/request" type="application/vnd.sap.sapgui" title="Z_BADI_CHECK - Testreport"/>
                                <atom:link adtcore:name="QE1K900019" href="/sap/bc/adt/cts/transportrequests/QE1K900019" rel="http://www.sap.com/adt/relations/transport/request" type="application/vnd.sap.adt.transportrequests.v1+xml" title="Z_BADI_CHECK - Testreport"/>
                                <atom:title>Z_BADI_CHECK - Testreport</atom:title>
                                <atom:updated>2019-12-06T17:53:09Z</atom:updated>
                            </atom:entry>
                            </atom:feed>
                            "#;
        let _result: VersionFeed = serde_xml_rs::from_str(plain).unwrap();
    }
}
