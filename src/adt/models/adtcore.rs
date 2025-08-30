use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "adtcore:packageRef")]
#[readonly::make]
pub struct PackageRef {
    #[serde(rename = "@adtcore:name")]
    pub name: String,

    #[serde(rename = "@adtcore:uri")]
    pub uri: String,

    #[serde(rename = "@adtcore:type")]
    pub object_type: String,
}
