/// Operations to manage objects, i.e locking / unlocking...
///
/// This works the same for programs, includes, classes, etc..
use derive_builder::Builder;
use http::{HeaderMap, HeaderValue, header};
use std::borrow::Cow;

use crate::{
    QueryParameters,
    models::asx::{self, LockResult},
    operation::{Operation, Stateful},
    response::Success,
};

// Possible actions to perform on objects
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectAction {
    Check,
    Activate,
    Lock,
    Unlock,
    Find,
}

/// Possible variants for objects that contain source code which can be modified.
#[derive(Debug, Clone)]
pub enum SourceCodeObject<'a> {
    Program(Cow<'a, str>),
    Include(Cow<'a, str>),
    GlobalClass(Cow<'a, str>),
    TestClass(Cow<'a, str>),
    Structure(Cow<'a, str>),
}

impl SourceCodeObject<'_> {
    pub fn object_uri(&self) -> String {
        match &self {
            Self::Program(name) => format!("/sap/bc/adt/programs/programs/{name}"),
            Self::GlobalClass(name) => format!("/sap/bc/adt/programs/includes/{name}"),
            Self::Include(name) => format!("/sap/bc/adt/programs/includes/{name}"),
            Self::TestClass(name) => format!("/sap/bc/adt/programs/includes/{name}"),
            Self::Structure(name) => format!("/sap/bc/adt/ddic/structures/{name}"),
        }
    }

    pub fn source_code_uri(&self) -> String {
        match &self {
            Self::Program(name) => format!("/sap/bc/adt/programs/programs/{name}/source/main"),
            Self::GlobalClass(name) => format!("/sap/bc/adt/programs/includes/{name}/source/main"),
            Self::Include(name) => format!("/sap/bc/adt/programs/includes/{name}/source/main"),
            Self::TestClass(name) => format!("/sap/bc/adt/programs/includes/{name}"),
            Self::Structure(name) => format!("/sap/bc/adt/ddic/structures/{name}/source/main"),
        }
    }
}

impl ObjectAction {
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Check => "CHECK",
            Self::Activate => "ACTIVATE",
            Self::Lock => "LOCK",
            Self::Unlock => "UNLOCK",
            Self::Find => "FIND",
        }
    }
}

/// Object access modes, not including ones used internally by ADT.
///
/// See `SEOK` typegroup on the ABAP System.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessMode {
    /// The object is locked but read-only and cannot be modified, to be confirmed.
    Show,
    /// The object is locked for modifications.
    Modify,
}

impl AccessMode {
    pub fn as_str(&self) -> &'static str {
        match &self {
            Self::Show => "SHOW",
            Self::Modify => "MODIFY",
        }
    }
}

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct Lock<'a> {
    /// The fully specified ADT URI of the object to unlock.
    /// ### Examples:
    /// - Classes: `classes/z_syntax_test`
    /// - Programs: `programs/programs`
    /// - Structures: `ddic/structures/zasupg_test_structure`
    #[builder(setter(into))]
    object_uri: Cow<'a, str>,

    access_mode: AccessMode,
}

impl Operation for Lock<'_> {
    const METHOD: http::Method = http::Method::POST;

    type Kind = Stateful;
    type Response = Success<asx::AsxData<LockResult>>;

    fn url(&self) -> Cow<'static, str> {
        Cow::Owned(self.object_uri.to_string())
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push("_action", ObjectAction::Lock.as_str());
        params.push("accessMode", self.access_mode.as_str());
        params
    }

    fn headers(&self) -> Option<http::HeaderMap> {
        let mut headers = http::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(
                "application/vnd.sap.as+xml; charset=utf-8; dataname=com.sap.adt.lock.Result2",
            ),
        );
        Some(headers)
    }
}

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct Unlock<'a> {
    /// The fully specified ADT URI of the object to unlock.
    /// ### Examples:
    /// - Classes: `/sap/bc/adt/oo/classes/z_syntax_test`
    /// - Programs: `/sap/bc/adt/programs/programs`
    /// - Structures: `/sap/bc/adt/ddic/structures/zasupg_test_structure`
    #[builder(setter(into))]
    object_uri: Cow<'a, str>,

    /// The lock handle that was obtained during the prior lock operation.
    #[builder(setter(into))]
    lock_handle: Cow<'a, str>,
}

impl Operation for Unlock<'_> {
    const METHOD: http::Method = http::Method::POST;

    type Kind = Stateful;
    type Response = Success<()>;

    fn url(&self) -> Cow<'static, str> {
        Cow::Owned(self.object_uri.to_string())
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push("_action", ObjectAction::Unlock.as_str());
        params.push("lockHandle", &self.lock_handle);
        params
    }
}

#[derive(Builder, Debug)]
#[builder(setter(strip_option))]
pub struct UpdateSourceCode<'a> {
    object: SourceCodeObject<'a>,

    #[builder(setter(into))]
    lock_handle: Cow<'a, str>,

    #[builder(setter(into))]
    content: Cow<'a, str>,
}

impl Operation for UpdateSourceCode<'_> {
    const METHOD: http::Method = http::Method::PUT;

    type Kind = Stateful;
    type Response = Success<()>;

    fn url(&self) -> Cow<'static, str> {
        self.object.source_code_uri().into()
    }

    fn parameters(&self) -> QueryParameters {
        let mut params = QueryParameters::default();
        params.push("lockHandle", &self.lock_handle);
        params
    }

    fn headers(&self) -> Option<http::HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        Some(headers)
    }

    fn body(&self) -> Option<Result<String, serde_xml_rs::Error>> {
        Some(Ok(self.content.clone().into_owned()))
    }
}
