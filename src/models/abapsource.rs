use crate::models::atom;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "abapsource:syntaxConfiguration")]
#[readonly::make]
pub struct SyntaxConfiguration {
    #[serde(rename = "abapsource:language")]
    pub language: Language,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "abapsource:language")]
#[readonly::make]
pub struct Language {
    #[serde(rename = "abapsource:version")]
    pub version: String,

    #[serde(rename = "abapsource:description")]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "abapsource:objectStructureElement")]
#[readonly::make]
pub struct ObjectStructureElement {
    /// Name of the object, for example `Z_BADI_CHECK`
    #[serde(rename = "@adtcore:name")]
    pub name: String,

    /// Type of the object e.g PROG/P for Programs
    #[serde(rename = "@adtcore:type")]
    pub kind: String,

    #[serde(rename = "atom:link")]
    pub link: atom::Link,

    #[serde(rename = "abapsource:objectStructureElement", default)]
    pub elements: Vec<Self>,
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn deserialize_program_object_structure() {
        let plain = r#"<abapsource:objectStructureElement xml:base="/sap/bc/adt/programs/programs/z_badi_check/source/main" adtcore:name="Z_BADI_CHECK" adtcore:type="PROG/P" xmlns:adtcore="http://www.sap.com/adt/core" xmlns:abapsource="http://www.sap.com/adt/abapsource" xmlns:atom="http://www.w3.org/2005/Atom">
                            <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/programs/z_badi_check"/>
                            <abapsource:objectStructureElement adtcore:name="T_T_PROTOCOL" adtcore:type="PROG/PY">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/includes/zbadicheck_inc/source/main?context=%2fsap%2fbc%2fadt%2fprograms%2fprograms%2fz_badi_check#type=PROG%2FPY;name=T_T_PROTOCOL"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="GC_ACTTYPE" adtcore:type="PROG/PD">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/programs/z_badi_check/source/main#type=PROG%2FPD;name=GC_ACTTYPE"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="GC_COMPONENT" adtcore:type="PROG/PD">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/programs/z_badi_check/source/main#type=PROG%2FPD;name=GC_COMPONENT"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="GC_COMPONENT_SAP_BASIS" adtcore:type="PROG/PD">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/includes/zbadicheck_inc/source/main?context=%2fsap%2fbc%2fadt%2fprograms%2fprograms%2fz_badi_check#type=PROG%2FPD;name=GC_COMPONENT_SAP_BASIS"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="INITIALIZE" adtcore:type="PROG/PU">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/programs/z_badi_check/source/main#type=PROG%2FPU;name=INITIALIZE"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="MAIN" adtcore:type="PROG/PU">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/programs/programs/z_badi_check/source/main#type=PROG%2FPU;name=MAIN"/>
                            </abapsource:objectStructureElement>
                            <abapsource:objectStructureElement adtcore:name="Z_BADI_CHECK" adtcore:type="PROG/PX">
                                <atom:link rel="http://www.sap.com/adt/relations/source/definitionIdentifier" href="/sap/bc/adt/textelements/programs/z_badi_check/source/symbols"/>
                            </abapsource:objectStructureElement>
                        </abapsource:objectStructureElement>"#;
        let result: ObjectStructureElement = serde_xml_rs::from_str(plain).unwrap();
        println!("{:?}", result);
    }
}
