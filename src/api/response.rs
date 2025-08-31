use std::ops::Deref;

use http::{self, StatusCode};
use serde::de::DeserializeOwned;
use thiserror::Error;

pub trait ResponseVariant: TryFrom<http::Response<String>, Error = ResponseError> {}
impl<T> ResponseVariant for T where T: TryFrom<http::Response<String>, Error = ResponseError> {}

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("unexpected status [{}]: {}", .0.status(), .0.body())]
    BadStatusCode(http::Response<String>),
    #[error(transparent)]
    ParseError(#[from] serde_xml_rs::Error),
}

#[derive(Debug)]
pub enum CacheControlled<T: DeserializeOwned> {
    Modified(http::Response<T>),
    NotModified(http::Response<()>),
}

impl<T> TryFrom<http::Response<String>> for CacheControlled<T>
where
    T: DeserializeOwned,
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
                    serde_xml_rs::from_str(&body)?,
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
