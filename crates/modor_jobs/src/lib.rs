//! Jobs crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_jobs = "0.1"
//! ```
//!
//! You can then use the components provided by this crate to start asynchronous jobs.

#[macro_use]
extern crate log;

mod asset_loading_job;
mod job;
mod platform;

pub use asset_loading_job::*;
pub use job::*;
pub use platform::*;
