#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[macro_use]
extern crate modor;

pub mod asset_loading;
pub mod job;
