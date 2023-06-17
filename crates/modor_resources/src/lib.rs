//! Resources module of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor = "0.1"
//! modor_jobs = "0.1"
//! ```
//!
//! You can then use the components provided by this crate to define and handle resources.

#[macro_use]
extern crate modor;
#[macro_use]
extern crate log;

mod components;
mod data;

pub mod testing;

pub use components::registry::*;
pub use data::handler::*;
pub use data::key::*;
