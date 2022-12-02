extern crate modor;

use modor::*;

struct Entity1;

#[singleton]
//~^ error: overflow evaluating the requirement `DependsOn<Entity1>: Sized`
impl Entity1 {
    #[run_after(Entity2)]
    fn f() {}
}

struct Entity2;

#[singleton]
impl Entity2 {
    #[run_after(Entity1)]
    fn f() {}
}

fn main() {}
