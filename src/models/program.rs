use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::models::{abapsource, adtcore, atom};

/// Represents an ABAP Program
#[derive(Debug, Deserialize)]
#[serde(rename = "program:abapProgram")]
#[readonly::make]
pub struct AbapProgram {
    /// The name of the program
    #[serde(rename = "@adtcore:name")]
    pub name: String,

    /// The object type of the program, should be `PROG/P`
    #[serde(rename = "@adtcore:type")]
    pub object_type: String,

    /// The datetime that the program was last changed at (UTC)
    #[serde(rename = "@adtcore:changedAt")]
    pub last_changed: DateTime<Utc>,

    /// The version of the program descriptor, e.g `active`
    #[serde(rename = "@adtcore:version")]
    pub version: String,

    /// The datetime that the program was created on (UTC)
    #[serde(rename = "@adtcore:createdAt")]
    pub created_at: DateTime<Utc>,

    /// The user who last changed this program
    #[serde(rename = "@adtcore:changedBy")]
    pub changed_by: String,

    /// The description of the program
    #[serde(rename = "@adtcore:description")]
    pub description: String,

    /// The character limit of the program description
    #[serde(rename = "@adtcore:descriptionTextLimit")]
    pub description_text_limit: i32,

    /// The language of the program, e.g. `EN`
    #[serde(rename = "@adtcore:language")]
    pub language: String,

    /// Whether the program is currently locked by the editor
    #[serde(rename = "@program:lockedByEditor")]
    pub locked_by_editor: bool,

    /// The type of program
    #[serde(rename = "@program:programType")]
    pub program_type: String,

    /// The relative uri to fetch the program source code
    #[serde(rename = "@abapsource:sourceUri")]
    pub source_uri: String,

    /// Whether the program supports fixed point arithmetic
    #[serde(rename = "@abapsource:fixPointArithmetic")]
    pub fix_point_arithmetic: bool,

    /// Whether unicode checks are active for this program
    #[serde(rename = "@abapsource:activeUnicodeCheck")]
    pub unicode_check_active: bool,

    /// The user who is responsible for this program
    #[serde(rename = "@adtcore:responsible")]
    pub responsible: String,

    /// Master language of the program
    #[serde(rename = "@adtcore:masterLanguage")]
    pub master_language: String,

    /// The system this program belongs to
    #[serde(rename = "@adtcore:masterSystem")]
    pub master_system: String,

    /// The ABAP Version of the program
    #[serde(rename = "@adtcore:abapLanguageVersion")]
    pub abap_language_version: String,

    /// Reference to the package the program belongs to
    #[serde(rename = "adtcore:packageRef")]
    pub package: adtcore::PackageRef,

    /// Syntax Configuration of the program
    #[serde(rename = "abapsource:syntaxConfiguration")]
    pub syntax_configuration: abapsource::SyntaxConfiguration,

    /// Relative URLs to related program endpoints
    #[serde(rename = "atom:link")]
    pub links: Vec<atom::Link>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_abap_program_data() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><program:abapProgram xmlns:program="http://www.sap.com/adt/programs/programs" program:lockedByEditor="false" program:programType="executableProgram" abapsource:sourceUri="source/main" abapsource:fixPointArithmetic="true" abapsource:activeUnicodeCheck="true" adtcore:responsible="DEVELOPER" adtcore:masterLanguage="EN" adtcore:masterSystem="A4H" adtcore:abapLanguageVersion="X" adtcore:name="ZWEGWERF1" adtcore:type="PROG/P" adtcore:changedAt="2025-08-30T21:49:44Z" adtcore:version="active" adtcore:createdAt="2023-03-08T00:00:00Z" adtcore:changedBy="DEVELOPER" adtcore:description="test" adtcore:descriptionTextLimit="70" adtcore:language="EN" xmlns:abapsource="http://www.sap.com/adt/abapsource" xmlns:adtcore="http://www.sap.com/adt/core">
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="source/main/versions" rel="http://www.sap.com/adt/relations/versions"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="source/main" rel="http://www.sap.com/adt/relations/source" type="text/plain" etag="202508302149440011"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="source/main" rel="http://www.sap.com/adt/relations/source" type="text/html" etag="202508302149440011"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="./zwegwerf1/objectstructure" rel="http://www.sap.com/adt/relations/objectstructure" type="application/vnd.sap.adt.objectstructure.v2+xml"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/textelements/programs/zwegwerf1" rel="http://www.sap.com/adt/relations/sources/textelements" title="Text Elements"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zwegwerf1/enhancements/implementations" rel="http://www.sap.com/adt/relations/enhancementImplementations" type="application/vnd.sap.adt.enhancementimplementations.v1+xml" title="Enhancement Implementations"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zwegwerf1/enhancements/options" rel="http://www.sap.com/adt/relations/enhancementOptionsOfMainObject" type="application/vnd.sap.adt.enhancementoptions.v2+xml" title="Enhancement Options of Main Object"/>
                        <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zwegwerf1/source/main/enhancements/options" rel="http://www.sap.com/adt/relations/enhancementOptions" type="application/vnd.sap.adt.enhancementoptions.v2+xml" title="Enhancement Options"/>
                        <adtcore:packageRef adtcore:uri="/sap/bc/adt/packages/%24tmp" adtcore:type="DEVC/K" adtcore:name="$TMP"/>
                        <abapsource:syntaxConfiguration>
                            <abapsource:language>
                            <abapsource:version>X</abapsource:version>
                            <abapsource:description>Standard ABAP</abapsource:description>
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/abapsource/parsers/rnd/grammar" rel="http://www.sap.com/adt/relations/abapsource/parser" type="text/plain" title="Standard ABAP" etag="757"/>
                            </abapsource:language>
                        </abapsource:syntaxConfiguration>
                        </program:abapProgram>
                        "#;

        let _result: AbapProgram = serde_xml_rs::from_str(plain).unwrap();
    }
}
