/// ASX (ABAP XML) - http://www.sap.com/abapxml
///
/// Essentially "uncategorized" models which do not fall under any special category.
/// These are usually wrapped in 'asx:abap' root elements but will be translated to
/// more intuitive structures to avoid name clashes and improve readability.
use serde::{Deserialize, Deserializer};
use std::ops::Deref;

/// Helper to wrap the inner type to be extracted from the unstructured
/// asx data and provide a deref to it.
#[derive(Debug, Deserialize)]
#[serde(rename = "asx:abap")]
pub struct AsxData<T> {
    #[serde(rename = "asx:values")]
    inner: AsxValuesInner<T>,
}

impl<T> AsxData<T> {
    pub fn inner(self) -> T {
        self.inner.data
    }
}

/// Internal helper to wrap the inner asx data
#[derive(Debug, Deserialize)]
struct AsxValuesInner<T> {
    #[serde(rename = "DATA")]
    data: T,
}

impl<T> Deref for AsxData<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner.data
    }
}

/// Contains the result of an object locking operation.
///
/// Content Type Version `com.sap.adt.lock.Result2``
#[derive(Debug, Deserialize)]
#[serde(rename = "DATA")]
#[readonly::make]
pub struct LockResult {
    /// Handle to the lock that was obtained, uniquely identifies the lock.
    #[serde(rename = "LOCK_HANDLE")]
    pub lock_handle: String,

    /// The number of the transport if the object is currently locked in one.
    #[serde(rename = "CORRNR")]
    pub transport_number: String,

    /// The owner of the transport if the object is currently locked in one.
    #[serde(rename = "CORRUSER")]
    pub transport_owner: String,

    /// The description of the transport if the object is currently locked in one.
    #[serde(rename = "CORRTEXT")]
    pub transport_description: String,

    /// Whether the object is a local development object.
    #[serde(rename = "IS_LOCAL", deserialize_with = "deserialize_abap_bool")]
    pub is_local: bool,

    /// To be clarified
    #[serde(rename = "IS_LINK_UP")]
    pub is_link_up: String,

    /// To be clarified
    #[serde(rename = "MODIFICATION_SUPPORT")]
    pub modification_support: String,

    /// To be clarified
    #[serde(rename = "LINK_UP_MODE")]
    pub link_up_mode: Option<String>,

    /// To be clarified
    #[serde(rename = "CORR_LOCKS")]
    pub corr_locks: Option<String>,

    /// To be clarified
    #[serde(rename = "CORR_CONTENTS")]
    pub corr_contents: Option<String>,

    /// To be clarified
    #[serde(rename = "SCOPE_MESSAGES")]
    pub scope_messages: String,
}

/// Deserialize `X` to `true` and all other values to `false`.
pub fn deserialize_abap_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    match s.as_str() {
        "X" => Ok(true),
        _ => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_local_lock_result() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?>
                    <asx:abap xmlns:asx="http://www.sap.com/abapxml" version="1.0">
                        <asx:values>
                            <DATA>
                            <LOCK_HANDLE>77D4511AAADBBE2691139283AA9D03A250C1FB22</LOCK_HANDLE>
                            <CORRNR/>
                            <CORRUSER/>
                            <CORRTEXT/>
                            <IS_LOCAL>X</IS_LOCAL>
                            <IS_LINK_UP/>
                            <MODIFICATION_SUPPORT>NoModification</MODIFICATION_SUPPORT>
                            <LINK_UP_MODE/>
                            <CORR_LOCKS/>
                            <CORR_CONTENTS/>
                            <SCOPE_MESSAGES/>
                            </DATA>
                        </asx:values>
                    </asx:abap>
                    "#;
        let result: AsxData<LockResult> = serde_xml_rs::from_str(&plain).unwrap();
        assert_eq!(result.is_local, true);
        assert_eq!(
            result.lock_handle,
            "77D4511AAADBBE2691139283AA9D03A250C1FB22"
        )
    }

    #[test]
    fn deserialize_lock_result_with_transport() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?>
                    <asx:abap xmlns:asx="http://www.sap.com/abapxml" version="1.0">
                        <asx:values>
                            <DATA>
                            <LOCK_HANDLE>2E84CAE23E14D343E405FC48FDBBFB3B28932EC4</LOCK_HANDLE>
                            <CORRNR>A4HK900089</CORRNR>
                            <CORRUSER>DEVELOPER</CORRUSER>
                            <CORRTEXT>Test Transport</CORRTEXT>
                            <IS_LOCAL/>
                            <IS_LINK_UP/>
                            <MODIFICATION_SUPPORT>ModificationsLoggedOnly</MODIFICATION_SUPPORT>
                            <LINK_UP_MODE/>
                            <CORR_LOCKS/>
                            <CORR_CONTENTS/>
                            <SCOPE_MESSAGES/>
                            </DATA>
                        </asx:values>
                    </asx:abap>
                    "#;
        let result: AsxData<LockResult> = serde_xml_rs::from_str(&plain).unwrap();
        assert_eq!(result.is_local, false);
        assert_eq!(result.transport_number, "A4HK900089");
    }
}
