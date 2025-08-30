use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "atom:link")]
#[readonly::make]
pub struct Link {
    #[serde(rename = "@href")]
    pub href: String,

    #[serde(rename = "@rel")]
    pub rel: String,

    #[serde(rename = "@type")]
    pub content: Option<String>,

    #[serde(rename = "@etag")]
    pub etag: Option<String>,

    #[serde(rename = "@title")]
    pub title: Option<String>,
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
                rel: String::from("http://www.sap.com/adt/relations/versions"),
                content: None,
                etag: None,
                title: None,
            },
        )
    }
}
