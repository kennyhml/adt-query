/// Facets - adt/ris/facets
use serde::Deserialize;

use crate::models::adtcomp::TemplateLink;

#[derive(Debug, Deserialize)]
#[serde(rename = "vf:facets")]
pub struct Facets {
    #[serde(rename = "vf:facet", default)]
    pub facets: Vec<Facet>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vf:facet")]
pub struct Facet {
    #[serde(rename = "@key")]
    pub key: String,

    #[serde(rename = "@displayName")]
    pub display_name: String,

    #[serde(rename = "@description")]
    pub description: String,

    #[serde(rename = "@isHierarchical")]
    pub is_hierarchical: bool,

    #[serde(rename = "@isForFiltering")]
    pub is_for_filtering: bool,

    #[serde(rename = "@isForStructuring")]
    pub is_for_structuring: bool,

    #[serde(rename = "adtcomp:templateLink")]
    pub values_uri: Option<TemplateLink>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_virtual_filesystem_facets() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><vf:facets xmlns:vf="http://www.sap.com/adt/ris/facets">
                        <vf:facet key="appl" displayName="Application Component" description="The application component of the development object." isHierarchical="true" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Application Components" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=appl{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="package" displayName="Package" description="The package to which the development object is assigned." isHierarchical="true" isForFiltering="true" isForStructuring="true"/>
                        <vf:facet key="group" displayName="Object Type Group" description="The group to which the type of the object belongs. Examples are dictionary or source code library." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Object Type Groups" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=group{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="type" displayName="Object Type" description="The four character object type of the development object. Alias types are used for non-unique transport types." isHierarchical="false" isForFiltering="true" isForStructuring="true"/>
                        <vf:facet key="owner" displayName="Owner" description="Usually the user who created the development object. Often it is also considered as the responsible user." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Owners" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=owner{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="api" displayName="API State" description="Development objects that were released as stable APIs for a dedicated purpose. The API state can be edited and displayed using the API State tab of the Properties view, or using the context menu entry of the Project Explorer." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="API States" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=api{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="language" displayName="Original Language" description="The original language of the development object." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Original Languages" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=language{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="system" displayName="Source System" description="The original system of a development object." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Source Systems" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=system{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        <vf:facet key="date" displayName="Creation Date" description="The day when the development object was created." isHierarchical="false" isForFiltering="true" isForStructuring="true">
                            <adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Creation Dates" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=date{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>
                        </vf:facet>
                        </vf:facets>
                        "#;

        let result: Facets = serde_xml_rs::from_str(plain).unwrap();
        println!("{:?}", result);
    }
}
