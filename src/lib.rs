pub mod auth;

pub mod endpoint;
pub mod error;
pub mod query;
pub mod response;

mod core;
pub use core::*;

#[cfg(feature = "adt")]
pub mod adt;

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder, ClientBuilderError};
