//! Resources crate of Modor.
//!
//! # Getting started
//!
//! You need to include these dependencies in your `Cargo.toml` file:
//! ```toml
//! modor_resources = "0.1"
//! ```
//!
//! Now you can start using this crate, for example by defining a [`Resource`].

mod resource;

pub use resource::*;

pub use modor;
