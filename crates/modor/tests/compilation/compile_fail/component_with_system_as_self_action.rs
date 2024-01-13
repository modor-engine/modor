extern crate modor;

use modor::*;

#[derive(Component)]
struct Component1;

#[systems]
impl Component1 {
    //~^ error: the trait bound `Component1Action: ComponentSystems` is not satisfied
    #[run_as(component(Self))]
    //~^ error: the trait bound `Component1Action: ComponentSystems` is not satisfied
    fn f() {}
}

fn main() {}
