#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[derive(Action)]
struct ActionWithNamedFields {}
//~^ error: structs with named fields cannot be actions

#[derive(Action)]
enum EnumAction {}
//~^ error: only structs can be actions
