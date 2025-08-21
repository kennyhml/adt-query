use serde::{Deserialize, Serialize};

use crate::api::{Endpoint, Stateless};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reporter {
    #[serde(rename = "@chkrun:name")]
    name: String,
    #[serde(rename = "chkrun:supportedType")]
    supported_types: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckReporters {
    #[serde(rename = "chkrun:reporter")]
    reporters: Vec<Reporter>,
}

pub struct Reporters {}

impl Endpoint for Reporters {
    type RequestBody = ();
    type ResponseBody = ();
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::GET;

    fn url(&self) -> std::borrow::Cow<'static, str> {
        "sap/bc/adt/checkruns/reporters".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_checkrun_reporters() {
        let plain_text = r#"<?xml version="1.0" encoding="UTF-8"?>
            <chkrun:checkReporters xmlns:chkrun="http://www.sap.com/adt/checkrun">
                <chkrun:reporter chkrun:name="abapCheckRunVersion-0">
                    <chkrun:supportedType>CLAS*</chkrun:supportedType>
                    <chkrun:supportedType>BDEF*</chkrun:supportedType>
                    <chkrun:supportedType>PROG*</chkrun:supportedType>
                </chkrun:reporter>
                <chkrun:reporter chkrun:name="abapCheckRunVersion-1">
                    <chkrun:supportedType>CLAS*</chkrun:supportedType>
                    <chkrun:supportedType>PROG*</chkrun:supportedType>
                </chkrun:reporter>
                <chkrun:reporter chkrun:name="abapCheckRunVersion-2">
                    <chkrun:supportedType>BDEF*</chkrun:supportedType>
                    <chkrun:supportedType>PROG*</chkrun:supportedType>
                </chkrun:reporter>
                <chkrun:reporter chkrun:name="abapCheckRunVersion-3">
                    <chkrun:supportedType>TYPE*</chkrun:supportedType>
                    <chkrun:supportedType>BDEF*</chkrun:supportedType>
                    <chkrun:supportedType>PROG*</chkrun:supportedType>
                </chkrun:reporter>
            </chkrun:checkReporters>"#;

        let result: CheckReporters = serde_xml_rs::from_str(plain_text).unwrap();
        assert_eq!(result.reporters.len(), 4, "Did not deserialize 4 reporters");
    }
}
