/// Virtual Filesystem Models (Virtual Folders, etc..) - adt/ris/virtualFolders
///
/// ABAP ADT Responsible: `CL_RIS_ADT_RES_VIRTUAL_FOLDERS`
use crate::adt::models::{adtcore, atom};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Collection of possible `Facet` values with a custom variant.
///
/// In the context of the Virtual Filesystem the facets serve
/// as a main filter / critera point to group objects together.
///
/// For example, facets can group together objects belonging to the same
/// owner, package or system.
///
/// Handled through `CE_VFS_FACET` on the server side.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Facet<'a> {
    Package,
    Group,
    Type,
    #[serde(rename = "OWNER")]
    Owner,
    #[serde(rename = "API")]
    ApiState,
    #[serde(rename = "COMP")]
    SoftwareComponent,
    #[serde(rename = "APPL")]
    ApplicationComponent,
    #[serde(rename = "LAYER")]
    TransportLayer,
    #[serde(rename = "FAV")]
    Favorites,
    Created,
    #[serde(rename = "MONTH")]
    CreationMonth,
    #[serde(rename = "DATE")]
    CreationDate,
    Language,
    #[serde(rename = "SYSTEM")]
    SourceSystem,
    Version,
    #[serde(rename = "MOD")]
    ModificationState,
    #[serde(rename = "DOCU")]
    Docu,
    #[serde(rename = "$value")]
    Custom(Cow<'a, str>),
}

// Need to handle serializing manually as serde_xml_rs refuses to just use the enum name as value.
// While quick_xml handles this correctly, it doesnt support namespaces properly.
impl<'a> Serialize for Facet<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Facet::Package => "PACKAGE",
            Facet::Group => "GROUP",
            Facet::Type => "TYPE",
            Facet::Owner => "OWNER",
            Facet::ApiState => "API",
            Facet::SoftwareComponent => "COMP",
            Facet::ApplicationComponent => "APPL",
            Facet::TransportLayer => "LAYER",
            Facet::Favorites => "FAV",
            Facet::Created => "CREATED",
            Facet::CreationMonth => "MONTH",
            Facet::CreationDate => "DATE",
            Facet::Language => "LANGUAGE",
            Facet::SourceSystem => "SYSTEM",
            Facet::Version => "VERSION",
            Facet::ModificationState => "MOD",
            Facet::Docu => "DOCU",
            Facet::Custom(val) => val.as_ref(),
        };
        serializer.serialize_str(s)
    }
}

/// Preselections represent object search filters, for example:
/// ```xml
/// <vfs:preselection facet="owner">
///     <vfs:value>DEVELOPER</vfs:value>
/// </vfs:preselection>
/// ```
/// Represents a filter for the facet `owner` with the value `DEVELOPER`. A value such as
/// `DEVELOPER` is included, whereas `-DEVELOPER` would be excluded from the selection.
///
/// On the AS ABAP, these are used by by the `CL_VFS_OBJECT_SELECTION` class to build
/// a select statement for selecting from `VFS_ALL`
///
/// When defining a package preselection, you can define a
///
/// TODO: Create an enum for the facet types that we already know with a `Custom` variant.
#[derive(Debug, Serialize, Clone, Builder)]
#[builder(setter(strip_option))]
#[serde(rename = "vfs:preselection")]
pub struct Preselection<'a> {
    /// The facet, i.e criteria, this filter applies to. For example `OWNER`, `PACKAGE`,
    /// `TYPE`, `GROUP`, `CREATED`..
    #[serde(rename = "@facet")]
    facet: Facet<'a>,

    /// The values that the facet is restricted to, this can either be included or excluded.
    ///
    /// **WARNING:** This does not appear to support patterns in the values.
    #[serde(rename = "vfs:value")]
    #[builder(setter(each(name = "include", into)), default)]
    values: Vec<Cow<'a, str>>,
}

impl<'a> PreselectionBuilder<'a> {
    /// Excludes the provided value from the preselection
    pub fn exclude(&mut self, value: &'a str) -> &mut Self {
        if !value.starts_with("-") {
            self.include(Cow::Owned(format!("-{value}")));
        } else {
            self.include(Cow::Borrowed(value));
        }
        self
    }
}

/// Information returned as part of a result that assists further queries in the hierarchy.
///
/// Based on the server code, this currently only supports facets of type `PACKAGE`.
#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:preselectionInfo")]
#[readonly::make]
pub struct PreselectionInfo<'a> {
    #[serde(rename = "@facet")]
    pub facet: Facet<'a>,

    #[serde(rename = "@hasChildrenOfSameFacet")]
    pub has_children_of_same_facet: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:virtualFolder")]
#[readonly::make]
pub struct VirtualFolder<'a> {
    /// Technical name of the folder, for example `INTF` for interfaces, `CLAS` for classes..
    #[serde(rename = "@name")]
    pub name: String,

    /// Display name of the folder, for example `Classes` or `Programs`
    #[serde(rename = "@displayName")]
    pub display_name: String,

    /// The kind of facet of the folder, e.g `GROUP` or `PACKAGE` or `TYPE`
    #[serde(rename = "@facet")]
    pub facet: Facet<'a>,

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
    #[builder(setter(each(name = "push")))]
    facets: Vec<Facet<'a>>,
}

impl<'a> From<Vec<Facet<'a>>> for FacetOrder<'a> {
    fn from(value: Vec<Facet<'a>>) -> Self {
        let mut facets = Vec::new();
        value.into_iter().for_each(|v| facets.push(v));
        FacetOrder { facets }
    }
}

#[derive(Debug, Serialize, Builder)]
#[serde(rename = "vfs:virtualFoldersRequest")]
#[builder(setter(strip_option))]
pub struct VirtualFoldersRequest<'a> {
    /// A search pattern that the object names must match. On the server side
    /// this is converted into a SQL pattern to query the objects with.
    #[serde(rename = "@objectSearchPattern")]
    #[builder(setter(into), default = Cow::Borrowed("*"))]
    search_pattern: Cow<'a, str>,

    /// Set of critera to filter the returned virtual folders with, see [`Preselection`]
    #[serde(rename = "vfs:preselection")]
    #[builder(setter(each(name = "preselection")), default)]
    preselections: Vec<Preselection<'a>>,

    /// The desired facets to be returned see, currently the server only seems
    /// to make use of the first value in the list.
    #[serde(rename = "vfs:facetorder")]
    #[builder(default)]
    order: FacetOrder<'a>,
}

/// Represents the result of a virtual folder query.
///
/// Mirrors `TS_VIRTUAL_FOLDERS_RESPONSE` of `CL_RIS_ADT_RES_VIRTUAL_FOLDERS`
#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:VirtualFoldersResult")]
pub struct VirtualFoldersResult<'a> {
    /// How many objects are part of the virtual folder
    #[serde(rename = "@objectCount")]
    pub object_count: i32,

    /// Only when a `package` preselection with a single, recursive value was specified.
    ///
    /// See [`PreselectionInfo`] for more information.
    #[serde(rename = "vfs:preselectionInfo")]
    pub preselection_info: Option<PreselectionInfo<'a>>,

    /// The virtual folders of the object we queried for
    #[serde(rename = "vfs:virtualFolder", default)]
    pub folders: Vec<VirtualFolder<'a>>,

    /// The sub-objects part of the object we queried for
    #[serde(rename = "vfs:object", default)]
    pub objects: Vec<Object>,

    /// Optional, links. To be clarified
    #[serde(rename = "atom:link", default)]
    pub links: Vec<atom::Link>,
}

/// Represents an object as part of a virtual folder.
///
/// Mirrors `TS_VIRTUAL_FOLDER_OBJECT` of `CL_RIS_ADT_RES_VIRTUAL_FOLDERS`
#[derive(Debug, Deserialize)]
#[serde(rename = "vfs:object")]
pub struct Object {
    /// Name of the object, for example `Z_CL_SOME_CLASS`
    #[serde(rename = "@name")]
    pub name: String,

    /// Optional: The version of the object
    #[serde(rename = "@version")]
    pub version: Option<adtcore::Version>,

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
    fn serialize_simple_preselection_filter() {
        let preselection = PreselectionBuilder::create_empty()
            .facet(Facet::Owner)
            .include("DEVELOPER")
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&preselection).unwrap();
        assert_eq!(
            result,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
            <vfs:preselection facet=\"OWNER\">\
                <vfs:value>DEVELOPER</vfs:value>\
            </vfs:preselection>"
        )
    }

    #[test]
    fn serialize_complex_preselection_filter() {
        let preselection = PreselectionBuilder::create_empty()
            .facet(Facet::ApplicationComponent)
            .include("foo")
            .include("bar")
            .exclude("baz")
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&preselection).unwrap();
        assert_eq!(
            result,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
            <vfs:preselection facet=\"APPL\">\
                <vfs:value>foo</vfs:value>\
                <vfs:value>bar</vfs:value>\
                <vfs:value>-baz</vfs:value>\
            </vfs:preselection>"
        )
    }

    #[test]
    fn serialize_known_facets() {
        let facets = vec![
            Facet::Package,
            Facet::Group,
            Facet::Type,
            Facet::Owner,
            Facet::ApiState,
            Facet::SoftwareComponent,
            Facet::ApplicationComponent,
            Facet::TransportLayer,
            Facet::Favorites,
            Facet::Created,
            Facet::CreationMonth,
            Facet::CreationDate,
            Facet::Language,
            Facet::SourceSystem,
            Facet::Version,
            Facet::ModificationState,
            Facet::Docu,
        ];
        let expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                            <vfs:facetorder>\
                            <vfs:facet>PACKAGE</vfs:facet>\
                            <vfs:facet>GROUP</vfs:facet>\
                            <vfs:facet>TYPE</vfs:facet>\
                            <vfs:facet>OWNER</vfs:facet>\
                            <vfs:facet>API</vfs:facet>\
                            <vfs:facet>COMP</vfs:facet>\
                            <vfs:facet>APPL</vfs:facet>\
                            <vfs:facet>LAYER</vfs:facet>\
                            <vfs:facet>FAV</vfs:facet>\
                            <vfs:facet>CREATED</vfs:facet>\
                            <vfs:facet>MONTH</vfs:facet>\
                            <vfs:facet>DATE</vfs:facet>\
                            <vfs:facet>LANGUAGE</vfs:facet>\
                            <vfs:facet>SYSTEM</vfs:facet>\
                            <vfs:facet>VERSION</vfs:facet>\
                            <vfs:facet>MOD</vfs:facet>\
                            <vfs:facet>DOCU</vfs:facet>\
                            </vfs:facetorder>";

        let xml = serde_xml_rs::to_string(&FacetOrder::from(facets)).unwrap();
        assert_eq!(xml, expected);
    }

    #[test]
    fn serialize_custom_facets() {
        let facets = vec![
            Facet::Custom("FOO".into()),
            Facet::Custom("BAR".into()),
            Facet::Custom("BAZ".into()),
        ];
        let expected = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
                            <vfs:facetorder>\
                            <vfs:facet>FOO</vfs:facet>\
                            <vfs:facet>BAR</vfs:facet>\
                            <vfs:facet>BAZ</vfs:facet>\
                            </vfs:facetorder>";

        let xml = serde_xml_rs::to_string(&FacetOrder::from(facets)).unwrap();
        assert_eq!(xml, expected);
    }

    #[test]
    fn serialize_virtual_folders_request() {
        let first_preselection = PreselectionBuilder::create_empty()
            .facet(Facet::Owner)
            .include("DEVELOPER")
            .include("JOHN DOE")
            .build()
            .unwrap();

        let second_preselection = PreselectionBuilder::create_empty()
            .facet(Facet::Package)
            .include("$TMP")
            .exclude("UI5/STRU")
            .build()
            .unwrap();

        let request = VirtualFoldersRequestBuilder::default()
            .preselection(first_preselection)
            .preselection(second_preselection)
            .order(
                FacetOrderBuilder::default()
                    .push(Facet::Owner)
                    .push(Facet::Package)
                    .push(Facet::Group)
                    .push(Facet::Type)
                    .build()
                    .unwrap(),
            )
            .build()
            .unwrap();

        let result = serde_xml_rs::to_string(&request).unwrap();
        assert_eq!(
            result,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
            <vfs:virtualFoldersRequest objectSearchPattern=\"*\">\
                <vfs:preselection facet=\"OWNER\">\
                    <vfs:value>DEVELOPER</vfs:value>\
                    <vfs:value>JOHN DOE</vfs:value>\
                </vfs:preselection>\
                <vfs:preselection facet=\"PACKAGE\">\
                    <vfs:value>$TMP</vfs:value>\
                    <vfs:value>-UI5/STRU</vfs:value>\
                </vfs:preselection>\
                <vfs:facetorder>\
                    <vfs:facet>OWNER</vfs:facet>\
                    <vfs:facet>PACKAGE</vfs:facet>\
                    <vfs:facet>GROUP</vfs:facet>\
                    <vfs:facet>TYPE</vfs:facet>\
                </vfs:facetorder>\
            </vfs:virtualFoldersRequest>"
        )
    }

    #[test]
    fn deserialize_virtual_folder_with_subfolders() {
        let plain = "<vfs:virtualFoldersResult xmlns:vfs=\"http://www.sap.com/adt/ris/virtualFolders\" objectCount=\"7\">\
                        <vfs:preselectionInfo facet=\"PACKAGE\" hasChildrenOfSameFacet=\"false\"/>\
                        <atom:link xmlns:atom=\"http://www.w3.org/2005/Atom\" href=\"/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20owner%3aDEVELOPER\" rel=\"http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection\" title=\"Virtual Folder Selection\"/>\
                        <vfs:virtualFolder hasChildrenOfSameFacet=\"false\" counter=\"2\" text=\"\" name=\"CLAS\" displayName=\"Classes\" facet=\"TYPE\">\
                            <atom:link xmlns:atom=\"http://www.w3.org/2005/Atom\" href=\"/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aCLAS%20owner%3aDEVELOPER\" rel=\"http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection\" title=\"Virtual Folder Selection\"/>\
                        </vfs:virtualFolder>\
                        <vfs:virtualFolder hasChildrenOfSameFacet=\"false\" counter=\"1\" text=\"\" name=\"INTF\" displayName=\"Interfaces\" facet=\"TYPE\">\
                            <atom:link xmlns:atom=\"http://www.w3.org/2005/Atom\" href=\"/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aINTF%20owner%3aDEVELOPER\" rel=\"http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection\" title=\"Virtual Folder Selection\"/>\
                        </vfs:virtualFolder>\
                        <vfs:virtualFolder hasChildrenOfSameFacet=\"false\" counter=\"4\" text=\"\" name=\"REPO\" displayName=\"Programs\" facet=\"APPL\">\
                            <atom:link xmlns:atom=\"http://www.w3.org/2005/Atom\" href=\"/sap/bc/adt/repository/informationsystem/virtualfolders?selection=package%3a%24TMP%20group%3aSOURCE_LIBRARY%20type%3aREPO%20owner%3aDEVELOPER\" rel=\"http://www.sap.com/adt/relations/informationsystem/virtualfolders/selection\" title=\"Virtual Folder Selection\"/>\
                        </vfs:virtualFolder>\
                    </vfs:virtualFoldersResult>";
        let result: VirtualFoldersResult = serde_xml_rs::from_str(plain).unwrap();
        assert_eq!(
            result.preselection_info.map(|v| v.facet),
            Some(Facet::Package)
        );
        assert_eq!(result.folders[2].facet, Facet::ApplicationComponent);
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
