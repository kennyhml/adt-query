use crate::error::QueryError;
use crate::{Contextualize, core::ContextId};
use async_trait::async_trait;
use http::{HeaderMap, HeaderValue};

#[async_trait]
pub trait StatelessQuery<T, R> {
    async fn query(&self, client: &T) -> Result<R, QueryError>;
}

#[async_trait]
pub trait StatefulQuery<T, R> {
    async fn query(&self, client: &T, context: ContextId) -> Result<R, QueryError>;
}

pub(crate) async fn inject_request_context<'a, S>(
    headers: &mut HeaderMap<HeaderValue>,
    session: &'a S,
    context_id: ContextId,
) -> Result<(), QueryError>
where
    S: Contextualize,
{
    headers.insert(
        "x-sap-adt-sessiontype",
        HeaderValue::from_static("stateful"),
    );

    if let Some(context_data) = session.context(context_id) {
        let cookie_header = headers
            .iter_mut()
            .find(|(k, _)| *k == "cookie")
            .map(|(_, v)| v);

        if let Some(value) = cookie_header {
            let as_string = value.to_str().unwrap();
            *value = HeaderValue::from_str(&format!(
                "{as_string}; {}",
                context_data.lock().await.cookie().as_cookie_pair()
            ))
            .unwrap();
        } else {
            return Err(QueryError::CookiesMissing);
        }
    } else {
        // Context has not yet been created on the server, in this case
        // setting the sessiontype header is good enough.
    }
    Ok(())
}
