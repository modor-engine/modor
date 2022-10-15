#![allow(
    clippy::struct_excessive_bools,
    clippy::unwrap_used,
    unused_tuple_struct_fields
)]

#[macro_use]
extern crate modor;

pub mod app;
pub mod entities;
pub mod ranges;
pub mod system_params;
pub mod system_runner;
