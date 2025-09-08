use http::header::InvalidHeaderValue;
use thiserror::Error;

use crate::api::ResponseError;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("xml serialization failed: {0}")]
    XmlError(#[from] serde_xml_rs::Error),
}

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("value for field '{0}' was not provided")]
    UninitializedField(&'static str),
}

#[derive(Debug, Error)]
pub enum BadRequest {
    #[error("invalid body: {0}")]
    SerializeError(#[from] SerializeError),
}

#[derive(Error, Debug)]
pub enum QueryError {
    #[error(transparent)]
    HttpError(#[from] http::Error),

    #[error(transparent)]
    BadRequest(#[from] BadRequest),

    #[error(transparent)]
    BadResponse(#[from] ResponseError),

    #[error("client is not authorized")]
    Unauthorized,

    #[error("cookies missing")]
    CookiesMissing,

    #[error("csrf-token missing for POST request")]
    MissingCsrfToken,

    #[error("bad url: {0}")]
    BadUrl(#[from] url::ParseError),

    #[error(transparent)]
    InvalidHeadervalue(#[from] InvalidHeaderValue),

    #[error("could not parse the body: {0}")]
    ParseError(#[from] serde_xml_rs::Error),

    #[error("unexpected response: {code} - {message}")]
    BadStatusCode {
        code: http::StatusCode,
        message: String,
    },

    #[error("invalid input: {0}")]
    BuilderError(#[from] BuilderError),

    #[cfg(feature = "client")]
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}
