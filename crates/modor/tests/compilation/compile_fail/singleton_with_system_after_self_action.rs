extern crate modor;

use modor::*;

struct Entity;

#[singleton]
//~^ error: the trait bound `EntityAction: modor::Component` is not satisfied
impl Entity {
    #[run_after(component(Self))]
    fn f() {}
}

fn main() {}
