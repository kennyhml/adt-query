use crate::error::{BadRequest, ResponseError, SerializeError};
use crate::query::{StatefulQuery, StatelessQuery};
use crate::{Client, QueryParameters, RequestDispatch};
use crate::{ContextId, error::QueryError};
use async_trait::async_trait;
use http::HeaderMap;
use http::request::Builder as RequestBuilder;
use std::borrow::Cow;

pub trait EndpointKind {}

pub struct Stateless {}
pub struct Stateful {}

impl EndpointKind for Stateful {}
impl EndpointKind for Stateless {}

/// An endpoint on the SAP System that can be called.
///
/// The implementing structure controls the request url, parameters and headers. Endpoints can
/// either be `Stateless` or `Stateful`. Check [`crate::core::Context`] for more information
/// on stateful endpoints.
///
/// In that sense, instances of endpoints can be seen as the prelude of a request to that endpoint.
///
/// The [`api::StatelessQuery`] or [`api::StatefulQuery`] traits are automatically implemented
/// for types that implement [`Endpoint`] depending on the associated `Kind` Type.
pub trait Endpoint {
    /// The type of response body of this endpoint, can be any deserializable structure or unit ().
    type Response: TryFrom<http::Response<String>, Error = ResponseError> + Send;

    /// The Kind of this endpoint, either [`Stateless`] or [`Stateful`] - marker type.
    type Kind: EndpointKind;

    /// The associated [`http::Method`] of this endpoint, e.g. `GET`, `POST`, `PUT`..
    const METHOD: http::Method;

    /// The relative URL for the query of this endpoint, outgoing from the system host.
    ///
    /// **Warning:** Use the [`parameters()`](method@parameters) method for query parameters.
    fn url(&self) -> Cow<'static, str>;

    /// The body to be included in the request, can be `None` if no body is desired. For more
    /// flexibility, this can be any string that is later passed into the request body.
    ///
    /// Remark: The request will inevitably have to clone the data anyway, so moving is fine.
    fn body(&self) -> Option<Result<String, SerializeError>> {
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

/// Any Endpoint where `Kind = Stateless` implements the `StatelessQuery` trait
#[async_trait]
impl<E, T> StatelessQuery<T, E::Response> for E
where
    E: Endpoint<Kind = Stateless> + Sync + Send,
    T: RequestDispatch,
{
    async fn query(&self, client: &Client<T>) -> Result<E::Response, QueryError> {
        let request = build_request(self, client)?;

        let body = self
            .body()
            .transpose()
            .map_err(BadRequest::SerializeError)?
            .unwrap_or_default();

        let response = client.dispatch_stateless(request, body).await?;
        Ok(E::Response::try_from(response)?)
    }
}

/// Any Endpoint where `Kind = Stateful` implements the `StatefulQuery` trait
#[async_trait]
impl<'a, E, T> StatefulQuery<T, E::Response> for E
where
    E: Endpoint<Kind = Stateful> + Sync + Send,
    T: RequestDispatch,
{
    async fn query(
        &self,
        client: &Client<T>,
        context: ContextId,
    ) -> Result<E::Response, QueryError> {
        let request = build_request(self, client)?;
        let body = self
            .body()
            .transpose()
            .map_err(BadRequest::SerializeError)?
            .unwrap_or_default();

        let response = client.dispatch_stateful(request, body, context).await?;
        Ok(E::Response::try_from(response)?)
    }
}

/// Helper method to build the fundamental request from an endpoint.
fn build_request<'a, T, E>(
    endpoint: &'a E,
    client: &'a Client<T>,
) -> Result<RequestBuilder, QueryError>
where
    T: RequestDispatch,
    E: Endpoint,
{
    let destination = client.destination();
    let mut uri = destination
        .server_url()
        .join("sap/bc/adt/")?
        .join(&endpoint.url())?;

    endpoint.parameters().add_to_url(&mut uri);

    let mut req = http::request::Builder::new()
        .method(E::METHOD)
        .uri(uri.as_str())
        .version(http::Version::HTTP_11);

    if let Some(headers) = endpoint.headers() {
        for (k, v) in headers.iter() {
            req = req.header(k, v);
        }
    }
    Ok(req)
}
