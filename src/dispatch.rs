use crate::error::OperationError;
use crate::session::UserSessionId;
use crate::{Client, RequestDispatch};
use async_trait::async_trait;

#[async_trait]
pub trait StatelessDispatch<T, R>
where
    T: RequestDispatch,
    R: Send,
{
    async fn dispatch(&self, client: &Client<T>) -> Result<R, OperationError>;
}

#[async_trait]
pub trait StatefulDispatch<T, R>
where
    T: RequestDispatch,
{
    async fn dispatch(&self, client: &Client<T>, ctx: UserSessionId) -> Result<R, OperationError>;
}
