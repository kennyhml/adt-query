use crate::auth::{AuthorizationKind, Credentials};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct Client {
    auth: AuthorizationKind,
    credentials: Credentials,
}

impl Client {}
