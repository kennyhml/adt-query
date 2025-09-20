/// Transport Properties (TPR) - http://www.sap.com/adt/ris/transportProperties
use crate::models::atom;
use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Represents the status of a Transport in the SAP System
///
/// Refer to domain `SCTS_REQ/TRSTATUS` in SAP System.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub enum TransportStatus {
    /// The transport is modifiable, objects may be added - default state.
    #[serde(rename = "D")]
    Modifiable,
    /// Ensures that only the owner of the transport can add more users.
    /// See [Protecting Transport Request](https://help.sap.com/docs/abap-cloud/abap-development-tools-user-guide/protecting-transport-request?locale=en-US)
    #[serde(rename = "L")]
    ProtectedModifiable,
    /// The release of the transport has started, details to be clarified.
    #[serde(rename = "O")]
    ReleaseStarted,
    /// The transport has been released.
    #[serde(rename = "R")]
    Released,
    /// The transport has been released but with import protection, to be clarified.
    #[serde(rename = "N")]
    ReleasedWithImportProtection,
    /// The transport is in preparation for release, to be clarified.
    #[serde(rename = "P")]
    ReleasePreparation,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "tpr:transportProperties")]
#[readonly::make]
pub struct TransportProperties {
    #[serde(rename = "tpr:transport", default)]
    pub transports: Vec<Transport>,
}

/// Represents an overview over a transport that an object is associated with.
#[derive(Debug, Deserialize)]
#[serde(rename = "tpr:transport")]
#[readonly::make]
pub struct Transport {
    /// Number (or ID) of the transport, e.g. `A4HK900089`
    #[serde(rename = "@number")]
    pub number: String,

    /// The description provided for the transport
    #[serde(rename = "@description")]
    pub description: String,

    /// The owner of the transport
    #[serde(rename = "@owner")]
    pub owner: String,

    /// The status of the transport, see [`TransportStatus`]
    #[serde(rename = "@status")]
    pub status: TransportStatus,

    /// The datetime the transport was created on
    #[serde(rename = "@createdAt")]
    pub created_at: DateTime<Utc>,

    /// The datetime the transport was last changed on
    #[serde(rename = "@changedAt")]
    pub last_changed: DateTime<Utc>,

    /// The number of entries in the transport, to be clarified how this differs from objects.
    #[serde(rename = "@numberOfEntries")]
    pub number_of_entries: i32,

    /// The number of objects which are part of this transport
    #[serde(rename = "@numberOfObjects")]
    pub number_of_objects: i32,

    /// Related links, e.g. a reference to the cts transport request.
    #[serde(rename = "atom:link")]
    pub links: Vec<atom::Link>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_transport_properties() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><tpr:transportProperties xmlns:tpr="http://www.sap.com/adt/ris/transportProperties">
                        <tpr:transport number="A4HK900089" description="Test Transport" owner="DEVELOPER" status="D" createdAt="2025-09-16T20:15:34Z" changedAt="2025-09-16T20:15:34Z" numberOfEntries="3" numberOfObjects="3">
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/cts/transportrequests/A4HK900089" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                        </tpr:transport>
                        </tpr:transportProperties>
                        "#;

        let result: TransportProperties = serde_xml_rs::from_str(&plain).unwrap();
        assert_eq!(result.transports.len(), 1, "Expected one transport");
        assert_eq!(result.transports[0].number, "A4HK900089");
        assert_eq!(result.transports[0].status, TransportStatus::Modifiable);
    }
}
