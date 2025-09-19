pub mod auth;

pub mod endpoint;
pub mod error;
pub mod query;
pub mod response;

mod client;
mod core;
pub use core::*;

pub mod adt;
pub use client::{Client, ClientBuilder, ClientBuilderError};
