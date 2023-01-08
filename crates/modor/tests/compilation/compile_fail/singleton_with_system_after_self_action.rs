extern crate modor;

use modor::*;

struct Entity;

#[singleton]
//~^ error: the trait bound `EntityAction: EntityMainComponent` is not satisfied
impl Entity {
    #[run_after(entity(Self))]
    fn f() {}
}

fn main() {}
