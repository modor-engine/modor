#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[derive(Action)]
struct MyAction {}
//~^ error: structs with named fields cannot be actions
