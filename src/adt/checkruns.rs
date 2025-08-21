use std::{borrow::Cow, ops::DerefMut};

use derive_builder::Builder;
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

#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkRunReports")]
pub struct CheckRunReports {
    #[serde(rename = "chkrun:checkReport")]
    reports: Vec<CheckReport>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkReport")]
pub struct CheckReport {
    #[serde(rename = "@chkrun:reporter")]
    reporter: String,

    #[serde(rename = "@chkrun:triggeringUri")]
    triggering_object_uri: String,

    #[serde(rename = "@chkrun:status")]
    status: String,

    #[serde(rename = "@chkrun:statusText")]
    status_text: String,

    #[serde(rename = "chkrun:checkMessageList")]
    messages: Option<CheckMessageList>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkMessageList")]
pub struct CheckMessageList {
    #[serde(rename = "chkrun:checkMessage")]
    messages: Vec<CheckMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkMessage")]
pub struct CheckMessage {
    #[serde(rename = "@chkrun:uri")]
    location_uri: String,

    #[serde(rename = "@chkrun:type")]
    kind: String,

    #[serde(rename = "@chkrun:shortText")]
    text: String,

    #[serde(rename = "atom:link")]
    quick_fix: Option<QuickFix>,
}

#[derive(Debug, Deserialize)]
pub struct QuickFix {
    #[serde(rename = "@href")]
    kind: String,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename = "chkrun:checkObjectList")]
pub struct CheckObjectList {
    #[serde(rename = "chkrun:checkObject")]
    objects: Vec<CheckObject>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CheckObject {
    #[serde(rename = "@adtcore:uri")]
    object_uri: String,

    #[serde(rename = "@chkrun:version")]
    version: String,
}

#[derive(Builder, Debug, Clone)]
pub struct RunCheck {
    objects: CheckObjectList,

    #[builder(setter(into))]
    reporter: String,
}

impl RunCheckBuilder {
    pub fn object<S>(&mut self, uri: S, version: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.objects
            .get_or_insert_default()
            .objects
            .push(CheckObject {
                object_uri: uri.into(),
                version: version.into(),
            });
        self
    }
}

impl Endpoint for RunCheck {
    type RequestBody = CheckObjectList;
    type ResponseBody = CheckRunReports;
    type Kind = Stateless;

    const METHOD: http::Method = http::Method::POST;

    fn url(&self) -> Cow<'static, str> {
        Cow::Owned(format!("sap/bc/adt/checkruns?reporters={}", self.reporter))
    }

    fn body(&self) -> Option<&Self::RequestBody> {
        Some(&self.objects)
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
    #[test]
    fn deserialize_check_report() {
        let plain_text = r#"<?xml version="1.0" encoding="UTF-8"?>
        <chkrun:checkRunReports xmlns:chkrun="http://www.sap.com/adt/checkrun">
            <chkrun:checkReport chkrun:reporter="abapCheckRun" chkrun:triggeringUri="/sap/bc/adt/oo/classes/z_syntax_test" chkrun:status="processed" chkrun:statusText="Object Z_SYNTAX_TEST has been checked">
                <chkrun:checkMessageList>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/oo/classes/z_syntax_test/source/main#start=193,19" chkrun:type="E" chkrun:shortText="Implementation missing for method &quot;CLS_METHODS_MULTIPLE1&quot;.">
                    <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="art.syntax:G(2" rel="http://www.sap.com/adt/categories/quickfixes"/>
                </chkrun:checkMessage>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/oo/classes/z_syntax_test/source/main#start=171,12" chkrun:type="E" chkrun:shortText="Implementation missing for method &quot;METHOD_WITH_SPECIAL_PARAMS&quot;.">
                    <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="art.syntax:G(2" rel="http://www.sap.com/adt/categories/quickfixes"/>
                </chkrun:checkMessage>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/oo/classes/z_syntax_test/source/main#start=184,18" chkrun:type="E" chkrun:shortText="Implementation missing for method &quot;SINGLE_CLS_METHOD&quot;.">
                    <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="art.syntax:G(2" rel="http://www.sap.com/adt/categories/quickfixes"/>
                </chkrun:checkMessage>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/oo/classes/z_syntax_test/source/main#start=178,12" chkrun:type="E" chkrun:shortText="Implementation missing for method &quot;SINGLE_METHOD_USING_ESCAPE&quot;.">
                    <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="art.syntax:G(2" rel="http://www.sap.com/adt/categories/quickfixes"/>
                </chkrun:checkMessage>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/functions/groups/http_runtime/fmodules/http_read_record/source/main#start=58,28" chkrun:type="W" chkrun:shortText="Use the addition &quot;USING CLIENT&quot; instead of &quot;CLIENT SPECIFIED&quot;."/>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/functions/groups/http_runtime/fmodules/http_read_debug/source/main#start=52,28" chkrun:type="W" chkrun:shortText="Use the addition &quot;USING CLIENT&quot; instead of &quot;CLIENT SPECIFIED&quot;."/>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/functions/groups/http_runtime/fmodules/http_read_debug/source/main#start=75,28" chkrun:type="W" chkrun:shortText="Use the addition &quot;USING CLIENT&quot; instead of &quot;CLIENT SPECIFIED&quot;."/>
                <chkrun:checkMessage chkrun:uri="/sap/bc/adt/functions/groups/http_runtime/fmodules/http_read_debug/source/main#start=96,35" chkrun:type="W" chkrun:shortText="Use the addition &quot;USING CLIENT&quot; instead of &quot;CLIENT SPECIFIED&quot;."/>
                </chkrun:checkMessageList>
            </chkrun:checkReport>
        </chkrun:checkRunReports>"#;

        let result: CheckRunReports = serde_xml_rs::from_str(plain_text).unwrap();
        assert_eq!(result.reports.len(), 1);
        assert_eq!(
            result.reports[0]
                .messages
                .as_ref()
                .map(|m| m.messages.len()),
            Some(8)
        );
    }
}
