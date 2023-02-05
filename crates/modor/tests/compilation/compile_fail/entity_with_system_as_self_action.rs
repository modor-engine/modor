extern crate modor;

use modor::*;

struct Entity;

#[entity]
//~^ error: the trait bound `EntityAction: modor::Component` is not satisfied
impl Entity {
    #[run_as(component(Self))]
    fn f() {}
}

fn main() {}
