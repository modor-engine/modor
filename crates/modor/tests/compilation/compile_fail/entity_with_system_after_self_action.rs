extern crate modor;

use modor::*;

struct Entity;

#[entity]
//~^ error: overflow evaluating the requirement `DependsOn<Entity>: Sized`
impl Entity {
    #[run_after(Self)]
    fn f() {}
}

fn main() {}
