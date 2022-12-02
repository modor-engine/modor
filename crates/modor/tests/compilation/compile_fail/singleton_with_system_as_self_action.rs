extern crate modor;

use modor::*;

struct Entity;

#[singleton]
//~^ error: overflow evaluating the requirement `DependsOn<Entity>: Sized`
impl Entity {
    #[run_as(Self)]
    fn f() {}
}

fn main() {}
