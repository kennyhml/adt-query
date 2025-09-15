/// Object Properties Models (OPR) - http://www.sap.com/adt/ris/objectProperties
///
/// Provides the data returned to descripe repository objects.
use crate::adt::models::{atom, vfs::Facet};
use serde::Deserialize;

/// Encapsulates the properties of a single object.
///
/// Contains a reference to the object of which the properties were fetched for
/// as well as a collection of the properties for the object.
///
///
#[derive(Debug, Deserialize)]
#[serde(rename = "opr:objectProperties")]
pub struct ObjectProperties {
    #[serde(rename = "opr:object")]
    object: Object,

    #[serde(rename = "opr:property")]
    properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "opr:object")]
pub struct Object {
    #[serde(rename = "@text")]
    text: String,

    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@package")]
    package: String,

    #[serde(rename = "@type")]
    kind: String,

    #[serde(rename = "@expandable")]
    expandable: bool,

    #[serde(rename = "atom:link")]
    links: Vec<atom::Link>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "opr:property")]
pub struct Property {
    #[serde(rename = "@facet")]
    facet: Facet,

    #[serde(rename = "@name")]
    name: String,

    #[serde(rename = "@displayName")]
    display_name: String,

    #[serde(rename = "@text")]
    text: Option<String>,

    #[serde(rename = "@hasChildrenOfSameFacet")]
    has_children_of_same_facet: Option<bool>,

    #[serde(rename = "atom:link", default)]
    links: Vec<atom::Link>,
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn object_properties_are_deserialized() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?>
                    <opr:objectProperties xmlns:opr="http://www.sap.com/adt/ris/objectProperties">
                        <opr:object text="URI Mapper" name="CL_ADT_URI_MAPPER" package="SADT_TOOLS_CORE" type="CLAS/OC" expandable="true">
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/oo/classes/cl_adt_uri_mapper" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/vit/wb/object_type/clasoc/object_name/CL_ADT_URI_MAPPER" rel="http://www.sap.com/adt/relations/objects" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                        </opr:object>
                        <opr:property facet="APPL" name="BC" displayName="BC" text="Basis Components" hasChildrenOfSameFacet="true"/>
                        <opr:property facet="APPL" name="BC-DWB" displayName="BC-DWB" text="ABAP Workbench, Java IDE and Infrastructure" hasChildrenOfSameFacet="true"/>
                        <opr:property facet="APPL" name="BC-DWB-AIE" displayName="BC-DWB-AIE" text="Installation and Infrastructure for ABAP Tools in Eclipse" hasChildrenOfSameFacet="true"/>
                        <opr:property facet="PACKAGE" name="BASIS" displayName="BASIS" text="BASIS Structure Package" hasChildrenOfSameFacet="true">
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/packages/basis" rel="http://www.sap.com/adt/relations/packages" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                        </opr:property>
                        <opr:property facet="PACKAGE" name="SADT_MAIN" displayName="SADT_MAIN" text="ABAP Development Tools" hasChildrenOfSameFacet="true">
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/packages/sadt_main" rel="http://www.sap.com/adt/relations/packages" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                        </opr:property>
                        <opr:property facet="PACKAGE" name="SADT_TOOLS_CORE" displayName="SADT_TOOLS_CORE" text="ADT Tools Core">
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/packages/sadt_tools_core" rel="http://www.sap.com/adt/relations/packages" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                        </opr:property>
                        <opr:property facet="GROUP" name="SOURCE_LIBRARY" displayName="Source Code Library"/>
                        <opr:property facet="TYPE" name="CLAS" displayName="Classes"/>
                        <opr:property facet="OWNER" name="SAP" displayName="SAP"/>
                        <opr:property facet="API" name="NOT_RELEASED" displayName="NOT_RELEASED"/>
                        <opr:property facet="LANGUAGE" name="EN" displayName="English"/>
                        <opr:property facet="SYSTEM" name="SAP" displayName="SAP"/>
                        <opr:property facet="VERSION" name="ACTIVE" displayName="ACTIVE" text="Active"/>
                        <opr:property facet="DOCU" name="SAPSCRIPT_POSSIBLE" displayName="SAP Script documentation possible"/>
                        <opr:property facet="CREATED" name="2009" displayName="2009"/>
                        <opr:property facet="MONTH" name="11" displayName="11" text="November"/>
                        <opr:property facet="DATE" name="20091126" displayName="26.11.2009" text="Thursday"/>
                    </opr:objectProperties>"#;

        let result: ObjectProperties = serde_xml_rs::from_str(plain).unwrap();
        assert_eq!(result.object.name, "CL_ADT_URI_MAPPER");
        assert_eq!(result.object.package, "SADT_TOOLS_CORE");
        assert_eq!(result.properties[0].facet, Facet::ApplicationComponent);
    }
}
