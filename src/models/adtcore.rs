use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::ParamValue;

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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Version {
    /// A persistent, active version of the workbench object
    #[serde(rename = "active")]
    Active,
    /// An inactive (modified, new...) object
    #[serde(rename = "inactive")]
    Inactive,
    /// The object is in the working area (to be clarified)
    #[serde(rename = "workingArea")]
    WorkingArea,
    /// The object is new (to be clarified)
    #[serde(rename = "new")]
    New,
    /// The object is partly active (to be clarified)
    #[serde(rename = "partly_act")]
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

impl<'a> ParamValue<'a> for Version {
    fn as_str(&self) -> Cow<'a, str> {
        Cow::Borrowed(self.as_str())
    }
}
