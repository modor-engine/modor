extern crate modor;

use modor::*;

struct Entity1;

#[entity]
impl Entity1 {
    //~^ error: overflow evaluating the requirement `Entity2Action: Sized`

    #[run_after(component(Entity2))]
    fn f() {}
}

struct Entity2;

#[entity]
impl Entity2 {
    #[run_after(component(Entity1))]
    fn f() {}
}

fn main() {}
