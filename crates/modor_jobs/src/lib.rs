//! Jobs module of modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_jobs = "0.1"
//! ```
//!
//! You can then use the components provided by this crate to start asynchronous jobs.

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;

pub use components::asset_loading::*;
pub use components::job::*;
