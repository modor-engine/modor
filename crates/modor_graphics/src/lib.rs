//! Graphics module of modor.

#[macro_use]
extern crate modor;
extern crate core;

#[macro_use]
mod utils;

mod appearance;
mod backend;
mod capture;
mod module;
mod runner;
mod storages;
mod window;

pub mod testing;

pub use appearance::*;
pub use capture::*;
pub use module::*;
pub use runner::*;
pub use window::*;
