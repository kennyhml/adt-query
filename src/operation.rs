use crate::dispatch::{StatefulDispatch, StatelessDispatch};
use crate::error::{OperationError, RequestError, ResponseError};
use crate::session::UserSessionId;
use crate::{Client, QueryParameters, RequestDispatch};
use async_trait::async_trait;
use http::HeaderMap;
use http::request::Builder as RequestBuilder;
use std::borrow::Cow;

pub trait OperationKind {}

pub struct Stateless {}
pub struct Stateful {}

impl OperationKind for Stateful {}
impl OperationKind for Stateless {}

/// An Operation on the SAP System that can be called.
///
/// The implementing structure controls the request url, parameters and headers. Operations can
/// either be `Stateless` or `Stateful`. Check [`crate::core::Context`] for more information
/// on stateful Operations.
///
/// In that sense, instances of Operations can be seen as the prelude of a request to that Operation.
///
/// The [`api::StatelessQuery`] or [`api::StatefulQuery`] traits are automatically implemented
/// for types that implement [`Operation`] depending on the associated `Kind` Type.
pub trait Operation {
    /// The type of response body of this Operation, can be any deserializable structure or unit ().
    type Response: TryFrom<http::Response<String>, Error = ResponseError> + Send;

    /// The Kind of this Operation, either [`Stateless`] or [`Stateful`] - marker type.
    type Kind: OperationKind;

    /// The associated [`http::Method`] of this Operation, e.g. `GET`, `POST`, `PUT`..
    const METHOD: http::Method;

    /// The relative URL for the query of this Operation, outgoing from the system host.
    ///
    /// **Warning:** Use the [`parameters()`](method@parameters) method for query parameters.
    fn url(&self) -> Cow<'static, str>;

    /// The body to be included in the request, can be `None` if no body is desired. For more
    /// flexibility, this can be any string that is later passed into the request body.
    ///
    /// Remark: The request will inevitably have to clone the data anyway, so moving is fine.
    fn body(&self) -> Option<Result<String, serde_xml_rs::Error>> {
        None
    }

    /// The query parameters to be added to the query.
    fn parameters(&self) -> QueryParameters {
        QueryParameters::default()
    }

    /// Extra headers to be included in the request, may be `None`.
    ///
    /// Common headers, such as session and context, are included independently.
    fn headers(&self) -> Option<HeaderMap> {
        None
    }
}

/// Any Operation where `Kind = Stateless` implements the `StatelessQuery` trait
#[async_trait]
impl<E, T> StatelessDispatch<T, E::Response> for E
where
    E: Operation<Kind = Stateless> + Sync + Send,
    T: RequestDispatch,
{
    async fn dispatch(&self, client: &Client<T>) -> Result<E::Response, OperationError> {
        let request = build_request(self, client)?;

        let body = self
            .body()
            .transpose()
            .map_err(RequestError::SerializeError)?
            .unwrap_or_default();

        let response = client.dispatch_stateless(request, body).await?;
        Ok(E::Response::try_from(response)?)
    }
}

/// Any Operation where `Kind = Stateful` implements the `StatefulQuery` trait
#[async_trait]
impl<'a, E, T> StatefulDispatch<T, E::Response> for E
where
    E: Operation<Kind = Stateful> + Sync + Send,
    T: RequestDispatch,
{
    async fn dispatch(
        &self,
        client: &Client<T>,
        ctx: UserSessionId,
    ) -> Result<E::Response, OperationError> {
        let request = build_request(self, client)?;
        let body = self
            .body()
            .transpose()
            .map_err(RequestError::SerializeError)?
            .unwrap_or_default();

        let response = client.dispatch_stateful(request, body, ctx).await?;
        Ok(E::Response::try_from(response)?)
    }
}

/// Helper method to build the fundamental request from an Operation.
fn build_request<'a, T, E>(
    operation_params: &'a E,
    client: &'a Client<T>,
) -> Result<RequestBuilder, RequestError>
where
    T: RequestDispatch,
    E: Operation,
{
    let destination = client.destination();
    let mut uri = destination
        .join("sap/bc/adt/")?
        .join(&operation_params.url())?;

    operation_params.parameters().add_to_url(&mut uri);

    let mut req = http::request::Builder::new()
        .method(E::METHOD)
        .uri(uri.as_str())
        .version(http::Version::HTTP_11);

    if let Some(headers) = operation_params.headers() {
        for (k, v) in headers.iter() {
            req = req.header(k, v);
        }
    }
    Ok(req)
}
