use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "adtcomp:templateLink")]
#[readonly::make]
pub struct TemplateLink {
    #[serde(rename = "@title")]
    pub title: String,

    #[serde(rename = "@rel")]
    pub relation: String,

    #[serde(rename = "@template")]
    pub template: String,

    #[serde(rename = "@type")]
    pub content_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_template_link() {
        let plain = r#"<adtcomp:templateLink xmlns:adtcomp="http://www.sap.com/adt/compatibility" title="Object Type Groups" rel="http://www.sap.com/adt/relations/informationsystem/propertyvalues" template="/sap/bc/adt/repository/informationsystem/properties/values?data=group{&amp;name}" type="application/vnd.sap.adt.nameditems.v1+xml"/>"#;

        let result: TemplateLink = serde_xml_rs::from_str(plain).unwrap();
        println!("{:?}", result);
    }
}
