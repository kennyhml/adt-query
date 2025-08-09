use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub struct StatefulSession {
    start: DateTime<Utc>,
    session_id: String,
}

pub struct StatelessSession {
    start: DateTime<Utc>,
    session_id: String,
}

pub trait SessionKind {}
pub trait Stateless: SessionKind {}
pub trait Stateful: SessionKind {}

// Could be either stateful or stateless
#[async_trait]
pub trait SessionRequest {
    async fn request(&self);
}

impl SessionKind for StatefulSession {}
impl Stateful for StatefulSession {}

impl SessionKind for StatelessSession {}
impl Stateless for StatelessSession {}
