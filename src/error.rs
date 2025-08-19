use http::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum QueryError {
    #[error("client is not authorized.")]
    Unauthorized,

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
}
