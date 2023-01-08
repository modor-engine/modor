extern crate modor;

use modor::*;

struct Entity;

#[entity]
//~^ error: the trait bound `EntityAction: EntityMainComponent` is not satisfied
impl Entity {
    #[run_as(entity(Self))]
    fn f() {}
}

fn main() {}
