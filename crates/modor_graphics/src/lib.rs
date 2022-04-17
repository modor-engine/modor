//! Graphics module of modor.

#[macro_use]
extern crate modor;
#[macro_use]
extern crate modor_internal;

mod backend;
mod components;
mod data;
mod entities;
mod runner;
mod storages;

pub mod testing;

pub use components::shape_color::*;
pub use data::*;
pub use entities::background_color::*;
pub use entities::module::*;
pub use entities::render_target::*;
pub use runner::*;
