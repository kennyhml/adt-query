pub mod auth;

pub mod common;
pub mod endpoint;
pub mod error;

mod core;
pub use core::*;

#[cfg(feature = "adt")]
pub mod adt;

#[cfg(feature = "client")]
mod client;

#[cfg(feature = "client")]
pub use client::{Client, ClientBuilder, ClientBuilderError};
