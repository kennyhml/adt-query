use std::borrow::Cow;

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

#[derive(Debug, Serialize)]
#[serde(rename = "chkrun:checkObjectList")]
pub struct CheckObjectList {
    #[serde(rename = "chkrun:checkObject")]
    objects: Vec<CheckObject>,
}

#[derive(Debug, Serialize)]
pub struct CheckObject {
    #[serde(rename = "@adtcore:uri")]
    object_uri: String,

    #[serde(rename = "@chkrun:version")]
    version: String,
}

pub struct RunCheck {
    object: String,
    reporter: String,
}

impl Endpoint for RunCheck {
    type RequestBody = ();
    type ResponseBody = ();
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::POST;

    fn url(&self) -> Cow<'static, str> {
        Cow::Owned(format!("sap/bc/adt/checkruns?reporters={}", self.reporter))
    }
}

pub struct Reporters {}

impl Endpoint for Reporters {
    type RequestBody = ();
    type ResponseBody = CheckReporters;
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

    #[test]
    fn serialize_check_objects() {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("chkrun", "http://www.sap.com/adt/checkrun")
            .namespace("adtcore", "http://www.sap.com/adt/core");

        let expected_result = r#"<?xml version="1.0" encoding="UTF-8"?><chkrun:checkObjectList xmlns:adtcore="http://www.sap.com/adt/core" xmlns:chkrun="http://www.sap.com/adt/checkrun"><chkrun:checkObject adtcore:uri="/sap/bc/adt/programs/programs/zwegwerf1" chkrun:version="active" /></chkrun:checkObjectList>"#;

        let content = CheckObjectList {
            objects: vec![CheckObject {
                object_uri: String::from("/sap/bc/adt/programs/programs/zwegwerf1"),
                version: String::from("active"),
            }],
        };

        let result: String = config.to_string(&content).unwrap();
        assert_eq!(result, expected_result);
    }
}
