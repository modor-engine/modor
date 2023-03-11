extern crate modor;

use modor::*;

#[derive(Component)]
struct Component1;

#[systems]
impl Component1 {
    //~^ error: overflow evaluating the requirement `Component2Action: Sized`

    #[run_after(component(Component2))]
    fn f() {}
}

#[derive(Component)]
struct Component2;

#[systems]
impl Component2 {
    #[run_after(component(Component1))]
    fn f() {}
}

fn main() {}
