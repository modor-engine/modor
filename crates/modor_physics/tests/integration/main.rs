#![allow(clippy::unwrap_used)]

#[macro_use]
extern crate modor;
#[macro_use]
extern crate modor_internal;

pub mod components;
pub mod entities;

struct TestEntity;

#[entity]
impl TestEntity {}
