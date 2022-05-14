#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[action]
struct TestAction;

struct InvalidEntity;

#[entity]
impl InvalidEntity {
    #[other]
    //~^ error: cannot find attribute `other` in this scope
    fn other_attribute() {}

    #[run::other]
    //~^ failed to resolve: use of undeclared crate or module `run`
    fn other_attribute_with_path() {}
}
