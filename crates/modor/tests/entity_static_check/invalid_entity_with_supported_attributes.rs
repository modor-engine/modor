#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[action]
struct TestAction;

struct InvalidEntity;

#[entity]
impl InvalidEntity {
    const CONSTANT: usize = 0;

    #[run]
    #[run]
    //~^ error: found more than one `run*` attribute
    fn multiple_run_attributes() {}

    #[run(TestAction)]
    //~^ error: expected syntax: `#[run]`
    fn wrong_run_attribute_syntax() {}

    #[run_as]
    //~^ error: expected syntax: `#[run_as(ActionType)]`
    fn wrong_run_as_attribute_syntax() {}

    #[run_as(TestAction, TestAction)]
    //~^ error: expected syntax: `#[run_as(ActionType)]`
    fn too_many_actions_passed_to_run_as_attribute() {}

    #[run_as("action")]
    //~^ error: expected syntax: `#[run_as(ActionType)]`
    fn literal_passed_to_run_as_attribute() {}

    #[run_after]
    //~^ error: expected syntax: `#[run_after(ActionType1, ActionType2, ...)]`
    fn wrong_run_after_attribute_syntax() {}

    #[run_after("action")]
    //~^ error: expected syntax: `#[run_after(ActionType1, ActionType2, ...)]`
    fn literal_passed_to_run_after_attribute() {}

    #[run_after_previous(TestAction)]
    //~^ error: expected syntax: `#[run_after_previous]`
    fn wrong_run_after_previous_attribute_syntax() {}
}
