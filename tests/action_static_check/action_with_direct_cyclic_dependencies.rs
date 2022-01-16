extern crate modor;

use modor::*;

struct Action1;

impl Action for Action1 {
    type Constraint = DependsOn<Action2>;
    //~^ error: overflow evaluating the requirement `modor::DependsOn<Action1>: Sized
}

struct Action2;

impl Action for Action2 {
    type Constraint = DependsOn<Action1>;
}

fn main() {}
