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
