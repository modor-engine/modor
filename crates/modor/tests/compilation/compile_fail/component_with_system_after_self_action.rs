extern crate modor;

use modor::*;

#[derive(Component)]
struct Component1;

#[systems]
//~^ error: the trait bound `Component1Action: ComponentSystems` is not satisfied
impl Component1 {
    #[run_after(component(Self))]
    fn f() {}
}

fn main() {}
