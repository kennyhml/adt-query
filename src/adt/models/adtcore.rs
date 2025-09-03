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

/// Reflects DDIC type `SADT_OBJ_VERSION` for object version management.
///
/// Is used for classes, programs and other objects alike. Documentation is lacking..
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum Version {
    /// A persistent, active version of the workbench object
    Active,
    /// An inactive (modified, new...) object
    Inactive,
    /// The object is in the working area (to be clarified)
    WorkingArea,
    /// The object is new (to be clarified)
    New,
    /// The object is partly active (to be clarified)
    PartlyActive,
}

impl Version {
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::WorkingArea => "workingArea",
            Self::New => "new",
            Self::PartlyActive => "partly_act",
        }
    }
}
