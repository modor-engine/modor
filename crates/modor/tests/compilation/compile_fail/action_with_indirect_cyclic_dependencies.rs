extern crate modor;

use modor::*;

struct Action1;

impl Action for Action1 {
    type Constraint = DependsOn<Action3>;
    //~^ error: overflow evaluating the requirement `modor::DependsOn<Action1>
}

struct Action2;

impl Action for Action2 {
    type Constraint = DependsOn<Action1>;
}

struct Action3;

impl Action for Action3 {
    type Constraint = DependsOn<Action2>;
}

fn main() {}
