use std::{borrow::Cow, ops::Deref};

use crate::error::ResponseError;
use http::{self, StatusCode};
use serde::de::DeserializeOwned;

/// A trait a type must implement to deserialize from a response body
pub trait DeserializeResponse {
    fn deserialize_response(body: String) -> Result<Self, ResponseError>
    where
        Self: Sized;
}

// Inherently, any type that can be deserialized, we can at least ATTEMPT
// to deserialize from the response body
impl<T> DeserializeResponse for T
where
    T: DeserializeOwned,
{
    fn deserialize_response(body: String) -> Result<Self, ResponseError> {
        serde_xml_rs::from_str(&body).map_err(ResponseError::ParseError)
    }
}

#[derive(Debug)]
pub enum CacheControlled<T: DeserializeResponse> {
    Modified(http::Response<T>),
    NotModified(http::Response<()>),
}

impl<T> TryFrom<http::Response<String>> for CacheControlled<T>
where
    T: DeserializeResponse,
{
    type Error = ResponseError;
    fn try_from(value: http::Response<String>) -> Result<Self, Self::Error> {
        match value.status() {
            StatusCode::NOT_MODIFIED => {
                // Drop the body from the response, there is no response body for this type.
                let (res, ..) = value.into_parts();
                Ok(Self::NotModified(http::Response::from_parts(res, ())))
            }
            StatusCode::OK => {
                // Deserialize to the expected response body
                let (res, body) = value.into_parts();
                Ok(Self::Modified(http::Response::from_parts(
                    res,
                    T::deserialize_response(body)?,
                )))
            }
            _ => Err(ResponseError::BadStatusCode(value)),
        }
    }
}

#[derive(Debug)]
pub struct Success<T: DeserializeOwned>(http::Response<T>);

impl<T> Deref for Success<T>
where
    T: DeserializeOwned,
{
    type Target = http::Response<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> TryFrom<http::Response<String>> for Success<T>
where
    T: DeserializeOwned,
{
    type Error = ResponseError;

    fn try_from(value: http::Response<String>) -> Result<Self, Self::Error> {
        match value.status() {
            StatusCode::OK => {
                let (res, body) = value.into_parts();
                Ok(Self(http::Response::from_parts(
                    res,
                    serde_xml_rs::from_str(&body)?,
                )))
            }
            _ => Err(ResponseError::BadStatusCode(value)),
        }
    }
}

/// Wraps a string-like type to bypass the xml parsing that happens as part
/// of the default deserialize behavior.
#[derive(Debug)]
pub struct Plain<'a>(Cow<'a, str>);

impl<'a> DeserializeResponse for Plain<'a> {
    fn deserialize_response(body: String) -> Result<Self, ResponseError> {
        Ok(Plain(Cow::Owned(body)))
    }
}

impl<'a> Deref for Plain<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
