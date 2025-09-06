/// Virtual Folders Models (Virtual Folders, etc..) - adt/ris/virtualFolders
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::adt::models::atom;

/// Preselections represent object search filters, for example:
/// ```xml
/// <vfs:preselection facet="owner">
///     <vfs:value>DEVELOPER</vfs:value>
/// </vfs:preselection>
/// ```
/// Represents a filter for the facet `owner`  with the value `DEVELOPER`.
#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename = "vfs:preselection")]
pub struct Preselection<'a> {
    #[serde(rename = "@facet")]
    facet: Cow<'a, str>,

    #[serde(rename = "vfs:value")]
    value: Cow<'a, str>,
}

impl<'a, T> Into<Preselection<'a>> for (T, T)
where
    T: Into<Cow<'a, str>>,
{
    fn into(self) -> Preselection<'a> {
        Preselection {
            facet: self.0.into(),
            value: self.1.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:preselectionInfo")]
#[readonly::make]
pub struct PreselectionInfo {
    #[serde(rename = "@facet")]
    pub facet: String,

    #[serde(rename = "@hasChildrenOfSameFacet")]
    pub has_children_of_same_facet: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:virtualFolder")]
#[readonly::make]
pub struct VirtualFolder {
    /// Technical name of the folder, for example `INTF` for interfaces, `CLAS` for classes..
    #[serde(rename = "@name")]
    pub name: String,

    /// Display name of the folder, for example `Classes` or `Programs`
    #[serde(rename = "@displayName")]
    pub display_name: String,

    /// The kind of facet of the folder, e.g `GROUP` or `PACKAGE` or `TYPE`
    #[serde(rename = "@facet")]
    pub facet: String,

    /// How many objects are contained in this folder in total
    #[serde(rename = "@counter")]
    pub object_count: i32,

    /// To be clarified
    #[serde(rename = "@text")]
    pub text: String,

    /// Whether the folder contains any folders of the same type.
    #[serde(rename = "@hasChildrenOfSameFacet")]
    pub has_children_of_same_facet: bool,

    /// Link to this folder, to be clarified how this can be used.
    #[serde(rename = "atom:link")]
    pub link: atom::Link,
}

#[derive(Debug, Serialize, Builder, Clone, Default)]
#[serde(rename = "vfs:facetorder")]
pub struct FacetOrder<'a> {
    #[serde(rename = "vfs:facet")]
    #[builder(setter(each(name = "push", into)))]
    facets: Vec<Cow<'a, str>>,
}

#[derive(Debug, Serialize, Builder)]
#[serde(rename = "vfs:virtualFoldersRequest")]
#[builder(setter(strip_option))]
pub struct VirtualFoldersRequest<'a> {
    #[serde(rename = "@objectSearchPattern")]
    #[builder(setter(into), default = Cow::Borrowed("*"))]
    search_pattern: Cow<'a, str>,

    #[serde(rename = "vfs:preselection")]
    #[builder(setter(each(name = "preselection", into)), default)]
    preselections: Vec<Preselection<'a>>,

    #[serde(rename = "vfs:facetorder")]
    #[builder(default)]
    order: FacetOrder<'a>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:VirtualFoldersResult")]
pub struct VirtualFoldersResult {
    #[serde(rename = "@objectCount")]
    pub object_count: i32,

    #[serde(rename = "vfs:preselectionInfo")]
    pub preselection_info: PreselectionInfo,

    #[serde(rename = "vfs:virtualFolder", default)]
    pub folders: Vec<VirtualFolder>,

    #[serde(rename = "vfs:object", default)]
    pub objects: Vec<Object>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:object")]
pub struct Object {
    /// Name of the object, for example `Z_CL_SOME_CLASS`
    #[serde(rename = "@name")]
    pub name: String,

    /// The name of the package the object is a part of
    #[serde(rename = "@package")]
    pub package: String,

    /// Technical type of the object, e.g `PROG/P` or `CLAS/OC`
    #[serde(rename = "@type")]
    pub kind: String,

    /// The uri of the object, generally this can be used to get information about the object
    #[serde(rename = "@uri")]
    pub uri: String,

    /// The URI of the object in the /vit/wb system. To be clarified
    #[serde(rename = "@vituri")]
    pub vituri: String,

    /// Whether the object supports being expanded into things it exposes or is grouped into
    #[serde(rename = "@expandable")]
    pub expandable: bool,

    // The description of the object
    #[serde(rename = "@text")]
    pub description: String,

    /// Related uris for the object that may be followed, in the case of vfs:object, this seems
    /// coincide with the `uri` and `vituri` attributes.
    #[serde(rename = "atom:link", default)]
    pub links: Vec<atom::Link>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_preselection_filter() {
        let preselection = Preselection {
            facet: "owner".into(),
            value: "DEVELOPER".into(),
        };

        let result = serde_xml_rs::to_string(&preselection).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:preselection facet="owner"><vfs:value>DEVELOPER</vfs:value></vfs:preselection>"#
        )
    }

    #[test]
    fn serialize_facet_order() {
        let order = FacetOrderBuilder::default()
            .push("owner")
            .push("package")
            .push("group")
            .push("type")
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&order).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:facetorder><vfs:facet>owner</vfs:facet><vfs:facet>package</vfs:facet><vfs:facet>group</vfs:facet><vfs:facet>type</vfs:facet></vfs:facetorder>"#
        )
    }

    #[test]
    fn serialize_virtual_folders_request() {
        let request = VirtualFoldersRequestBuilder::default()
            .preselection(("owner", "DEVELOPER"))
            .preselection(("package", "$TMP"))
            .order(
                FacetOrderBuilder::default()
                    .push("owner")
                    .push("package")
                    .push("group")
                    .push("type")
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&request).unwrap();
        assert_eq!(
            result,
            r#"<?xml version="1.0" encoding="UTF-8"?><vfs:virtualFoldersRequest objectSearchPattern="*"><vfs:preselection facet="owner"><vfs:value>DEVELOPER</vfs:value></vfs:preselection><vfs:preselection facet="package"><vfs:value>$TMP</vfs:value></vfs:preselection><vfs:facetorder><vfs:facet>owner</vfs:facet><vfs:facet>package</vfs:facet><vfs:facet>group</vfs:facet><vfs:facet>type</vfs:facet></vfs:facetorder></vfs:virtualFoldersRequest>"#
        )
    }

    #[test]
    fn deserialize_virtual_folder_with_subfolders() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><vfs:virtualFoldersResult xmlns:vfs="http://www.sap.com/adt/ris/virtualFolders" objectCount="7">
                            <vfs:preselectionInfo facet="PACKAGE" hasChildrenOfSameFacet="false"/>
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20owner%3aDEVELOPER" rel="http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection" title="Virtual Folder Selection"/>
                            <vfs:virtualFolder hasChildrenOfSameFacet="false" counter="2" text="" name="CLAS" displayName="Classes" facet="TYPE">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aCLAS%20owner%3aDEVELOPER" rel="http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection" title="Virtual Folder Selection"/>
                            </vfs:virtualFolder>
                            <vfs:virtualFolder hasChildrenOfSameFacet="false" counter="1" text="" name="INTF" displayName="Interfaces" facet="TYPE">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aINTF%20owner%3aDEVELOPER" rel="http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection" title="Virtual Folder Selection"/>
                            </vfs:virtualFolder>
                            <vfs:virtualFolder hasChildrenOfSameFacet="false" counter="4" text="" name="REPO" displayName="Programs" facet="TYPE">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aREPO%20owner%3aDEVELOPER" rel="http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection" title="Virtual Folder Selection"/>
                            </vfs:virtualFolder>
                            </vfs:virtualFoldersResult>
                            "#;
        let result: VirtualFoldersResult = serde_xml_rs::from_str(plain).unwrap();
        assert_eq!(result.preselection_info.facet, "PACKAGE");
    }

    #[test]
    fn deserialize_virtual_folder_with_objects() {
        let plain = r#"<?xml version="1.0" encoding="UTF-8"?><vfs:virtualFoldersResult xmlns:vfs="http://www.sap.com/adt/ris/virtualFolders" objectCount="4">
                            <vfs:preselectionInfo facet="PACKAGE" hasChildrenOfSameFacet="false"/>
                            <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aREPO%20owner%3aDEVELOPER" rel="http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection" title="Virtual Folder Selection"/>
                            <vfs:object uri="/sap/bc/adt/programs/programs/zabapgit_standalone" vituri="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZABAPGIT_STANDALONE" text="Zabapgit_Standalone" name="ZABAPGIT_STANDALONE" package="$TMP" type="PROG/P" expandable="true">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zabapgit_standalone" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZABAPGIT_STANDALONE" rel="http://www.sap.com/adt/relations/objects" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                            </vfs:object>
                            <vfs:object uri="/sap/bc/adt/programs/programs/zdemo1" vituri="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZDEMO1" text="test" name="ZDEMO1" package="$TMP" type="PROG/P" expandable="true">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zdemo1" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZDEMO1" rel="http://www.sap.com/adt/relations/objects" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                            </vfs:object>
                            <vfs:object uri="/sap/bc/adt/programs/programs/zwegwerf1" vituri="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZWEGWERF1" text="test" name="ZWEGWERF1" package="$TMP" type="PROG/P" expandable="true">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/zwegwerf1" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/vit/wb/object_type/progp/object_name/ZWEGWERF1" rel="http://www.sap.com/adt/relations/objects" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                            </vfs:object>
                            <vfs:object uri="/sap/bc/adt/programs/programs/z_abapgit_standalone_20_03" vituri="/sap/bc/adt/vit/wb/object_type/progp/object_name/Z_ABAPGIT_STANDALONE_20_03" text="Z_ABAPGIT_Standalone_20_03" name="Z_ABAPGIT_STANDALONE_20_03" package="$TMP" type="PROG/P" expandable="true">
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/programs/programs/z_abapgit_standalone_20_03" rel="http://www.sap.com/adt/relations/objects" title="ADT Object Reference"/>
                                <atom:link xmlns:atom="http://www.w3.org/2005/Atom" href="/sap/bc/adt/vit/wb/object_type/progp/object_name/Z_ABAPGIT_STANDALONE_20_03" rel="http://www.sap.com/adt/relations/objects" type="application/vnd.sap.sapgui" title="ADT Object Reference"/>
                            </vfs:object>
                            </vfs:virtualFoldersResult>"#;
        let result: VirtualFoldersResult = serde_xml_rs::from_str(plain).unwrap();
        assert_eq!(
            result.objects.iter().filter(|o| o.kind == "PROG/P").count(),
            4,
            "Expected 4 PROG/P objects in the virtual folder result."
        );
    }
}
