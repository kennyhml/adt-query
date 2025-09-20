/// Endpoints to manage objects, i.e locking / unlocking...
///
/// This works the same for programs, includes, classes, etc..
use derive_builder::Builder;
use http::{HeaderValue, Response, StatusCode, header};
use std::borrow::Cow;

use crate::{
    QueryParameters,
    endpoint::{Endpoint, Stateful},
    error::ResponseError,
    models::asx,
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

#[derive(Debug)]
pub enum LockResult {
    /// Object is already locked, response body contains who has locked the object
    AlreadyLocked(http::Response<()>),

    /// Object was locked successfully
    ObjectLocked(http::Response<asx::LockResult>),
}

impl TryFrom<http::Response<String>> for LockResult {
    type Error = ResponseError;

    fn try_from(value: http::Response<String>) -> Result<Self, Self::Error> {
        let (parts, body) = value.into_parts();

        let result = match parts.status {
            StatusCode::OK => {
                let asx: asx::AsxData<asx::LockResult> = serde_xml_rs::from_str(&body)?;
                Self::ObjectLocked(Response::from_parts(parts, asx.inner()))
            }
            StatusCode::FORBIDDEN => Self::AlreadyLocked(Response::from_parts(parts, ())),
            _ => Err(ResponseError::BadStatusCode(Response::from_parts(
                parts, body,
            )))?,
        };
        Ok(result)
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

impl Endpoint for Lock<'_> {
    const METHOD: http::Method = http::Method::POST;

    type Kind = Stateful;
    type Response = LockResult;

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

impl Endpoint for Unlock<'_> {
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
