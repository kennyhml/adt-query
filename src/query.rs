use crate::error::QueryError;
use crate::session::UserSessionId;
use crate::{Client, RequestDispatch};
use async_trait::async_trait;

#[async_trait]
pub trait StatelessQuery<T, R>
where
    T: RequestDispatch,
    R: Send,
{
    async fn query(&self, client: &Client<T>) -> Result<R, QueryError>;
}

#[async_trait]
pub trait StatefulQuery<T, R>
where
    T: RequestDispatch,
{
    async fn query(&self, client: &Client<T>, ctx: UserSessionId) -> Result<R, QueryError>;
}
