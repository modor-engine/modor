#![allow(
    clippy::struct_excessive_bools,
    clippy::unwrap_used,
    unused_tuple_struct_fields
)]

#[macro_use]
extern crate modor;

pub mod app;
pub mod changed_filter;
pub mod entities;
pub mod entity_actions;
pub mod filters;
pub mod indirect_actions;
pub mod ranges;
pub mod system_params;
pub mod system_runner;
