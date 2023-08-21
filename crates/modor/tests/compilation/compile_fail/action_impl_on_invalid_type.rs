#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[derive(Action, Clone, Copy)]
struct ValidAction;

#[derive(Action)]
enum EnumAction {}
//~^ error: action cannot be an enum

#[derive(Action)]
union UnionAction {
    //~^ error: action cannot be a union
    action: ValidAction,
}
