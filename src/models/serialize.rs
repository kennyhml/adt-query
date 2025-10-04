use serde::Serialize;
use std::borrow::Cow;

pub trait IntoXmlRoot: Serialize {
    fn namespaces(&self) -> Vec<(Cow<'static, str>, Cow<'static, str>)>;

    fn into_xml_root(&self) -> Result<String, serde_xml_rs::Error> {
        let mut serializer = serde_xml_rs::SerdeXml::new();
        for (k, v) in self.namespaces().into_iter() {
            serializer = serializer.namespace(k, v);
        }
        Ok(serializer.to_string(&self)?)
    }
}
