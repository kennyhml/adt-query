use http::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("unexpected status [{}]: {}", .0.status(), .0.body())]
    BadStatusCode(http::Response<String>),
    #[error(transparent)]
    DeserializeError(#[from] serde_xml_rs::Error),
}

#[derive(Debug, Error)]
pub enum RequestError {
    #[error(transparent)]
    BadHeader(#[from] InvalidHeaderValue),

    #[error(transparent)]
    SerializeError(#[from] serde_xml_rs::Error),

    #[error("invalid url: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

/// Something went wrong with dispatching the request to the backend.
#[derive(Error, Debug)]
pub enum DispatchError {
    #[error(transparent)]
    HttpError(#[from] http::Error),

    #[error("the target machine actively refused the connection.")]
    ConnectionRefused,

    #[cfg(feature = "reqwest")]
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    #[error("bad url: {0}")]
    BadUrl(#[from] url::ParseError),
}

/// The request could not be dispatched because the operation was not
/// built in a way that is valid to dispatch, or the request was dispatched
/// successfully but the response is not one of the expected responses.
#[derive(Error, Debug)]
pub enum OperationError {
    #[error(transparent)]
    BadResponse(#[from] ResponseError),

    #[error(transparent)]
    BadRequest(#[from] RequestError),

    #[error(transparent)]
    DispatchError(#[from] DispatchError),

    #[error("value for field '{0}' was not provided")]
    UninitializedField(&'static str),
}
