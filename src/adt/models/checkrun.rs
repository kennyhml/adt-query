use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// A `Reporter` that can be used to check objects.
///
/// Provides the name and supported object types of the reporter.
#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:reporter")]
#[readonly::make]
pub struct Reporter {
    /// The name of the reporter used to adress it, e.g. `abapCheckRun`.
    #[serde(rename = "@chkrun:name")]
    pub name: String,

    /// What objects this reporter can be used on
    #[serde(rename = "chkrun:supportedType")]
    pub supported_types: Vec<String>,
}

/// Wraps a collection of [`Reporter`]
///
/// Typically the root element of the related XML Response.
#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkReporters")]
#[readonly::make]
pub struct Reporters {
    #[serde(rename = "chkrun:reporter")]
    pub reporters: Vec<Reporter>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkReport")]
#[readonly::make]
pub struct Report {
    /// The name of the [`Reporter`] that was used for the check.
    #[serde(rename = "@chkrun:reporter")]
    pub reporter: String,

    /// The object that triggered the check.
    #[serde(rename = "@chkrun:triggeringUri")]
    pub object_uro: String,

    /// The status of the check, e.g `Processed`.
    #[serde(rename = "@chkrun:status")]
    pub status: String,

    /// A long status text of the check, e.g, `"The object has been processed."`.
    #[serde(rename = "@chkrun:statusText")]
    pub status_text: String,

    /// Optional, a collection of [`Message`]s relevant to the check.
    #[serde(rename = "chkrun:checkMessageList")]
    pub messages: Option<MessageList>,
}

/// Wraps a collection of [`Report`]
///
/// Typically the root element of the related XML Response.
#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkRunReports")]
#[readonly::make]
pub struct Reports {
    #[serde(rename = "chkrun:checkReport")]
    pub reports: Vec<Report>,
}

/// A message relevant to the check of an object.
///
/// Provides the location the message refers to, the type, text and possibly a fix.
#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkMessage")]
#[readonly::make]
pub struct Message {
    /// The location the message refers to in the source code (where the problem occurs).
    #[serde(rename = "@chkrun:uri")]
    pub location_uri: String,

    /// The kind of message, e.g `W` for **Warning**, or `E` for **Error**.
    #[serde(rename = "@chkrun:type")]
    pub kind: String,

    /// An informational text about what the "problem" or reason of the message is.
    #[serde(rename = "@chkrun:shortText")]
    pub text: String,

    /// Optional: a quickfix to the problem at hand.
    #[serde(rename = "atom:link")]
    pub quick_fix: Option<QuickFix>,
}

/// Wraps a collection of [`Message`]s.
///
/// Typically the root element of the related XML Response.
#[derive(Debug, Deserialize)]
#[serde(rename = "chkrun:checkMessageList")]
#[readonly::make]
pub struct MessageList {
    #[serde(rename = "chkrun:checkMessage")]
    pub messages: Vec<Message>,
}

/// A quick fix to an error or warning in the code.
///
/// Seems to refer to some kind e.g `art.syntax2G(`.
///
/// TODO: Find out how this works.
#[derive(Debug, Deserialize)]
#[readonly::make]
pub struct QuickFix {
    #[serde(rename = "@href")]
    kind: String,
}

/// An object to be checked by the check runner.
///
/// Provides the uri to the object to check as well as the version.
///
/// ## Example:
/// ```
/// ObjectBuilder::default()
///     .object_uri("/sap/bc/adt/programs/programs/z_my_program")
///     .version("active")
///     .build()
/// ```
#[derive(Builder, Debug, Serialize, Clone)]
#[serde(rename = "chkrun:checkObject")]
pub struct Object {
    #[serde(rename = "@adtcore:uri")]
    #[builder(setter(into))]
    object_uri: String,

    #[serde(rename = "@chkrun:version")]
    #[builder(setter(into))]
    version: String,
}

/// Wraps a collection of [`Object`]
///
/// Typically the root element of a XML Body.
#[derive(Builder, Debug, Serialize, Clone, Default)]
#[serde(rename = "chkrun:checkObjectList")]
pub struct ObjectList {
    #[serde(rename = "chkrun:checkObject")]
    #[builder(setter(each(name = object)))]
    objects: Vec<Object>,
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

        let result: Reporters = serde_xml_rs::from_str(plain_text).unwrap();
        assert_eq!(result.reporters.len(), 4, "Did not deserialize 4 reporters");
    }

    #[test]
    fn serialize_check_objects() {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("chkrun", "http://www.sap.com/adt/checkrun")
            .namespace("adtcore", "http://www.sap.com/adt/core");

        let expected_result = r#"<?xml version="1.0" encoding="UTF-8"?><chkrun:checkObjectList xmlns:adtcore="http://www.sap.com/adt/core" xmlns:chkrun="http://www.sap.com/adt/checkrun"><chkrun:checkObject adtcore:uri="/sap/bc/adt/programs/programs/zwegwerf1" chkrun:version="active" /></chkrun:checkObjectList>"#;

        let content = ObjectList {
            objects: vec![Object {
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

        let result: Reports = serde_xml_rs::from_str(plain_text).unwrap();
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
