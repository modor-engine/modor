#[macro_use]
extern crate modor;
extern crate core;

use modor::*;

fn main() {}

#[derive(Action)]
struct TestAction;

#[derive(Component)]
struct InvalidComponent;

#[systems]
impl InvalidComponent {
    const CONSTANT: usize = 0;

    #[run]
    #[run]
    //~^ error: found more than one `run*` attribute
    fn multiple_run_attributes() {}

    #[run(TestAction)]
    //~^ error: Unexpected meta-item format `list`
    fn wrong_run_attribute_syntax() {}

    #[run_as]
    //~^ error: Unexpected meta-item format `word`
    fn wrong_run_as_attribute_syntax() {}

    #[run_as()]
    //~^ error: Too few items: Expected at least 1
    fn too_few_actions_passed_to_run_as_attribute() {}

    #[run_as(TestAction, TestAction)]
    //~^ error: Too many items: Expected no more than 1
    fn too_many_actions_passed_to_run_as_attribute() {}

    #[run_as("action")]
    //~^ error: Unexpected meta-item format `literal`
    fn literal_passed_to_run_as_attribute() {}

    #[run_as(entity::attribute(MySingleton))]
    //~^ error: Unknown field: `entity::attribute`
    fn sub_attribute_with_multiple_parts_passed_to_run_as_attribute() {}

    #[run_as(singleton(MySingleton))]
    //~^ error: Unknown field: `singleton`
    fn unknown_sub_attribute_passed_to_run_as_attribute() {}

    #[run_as(component(MySingleton, MySingleton))]
    //~^ error: expected exactly one type
    fn multiple_entities_in_same_component_sub_attribute_passed_to_run_as_attribute() {}

    #[run_as(action(TestAction, TestAction))]
    //~^ error: expected exactly one type
    fn multiple_entities_in_same_action_sub_attribute_passed_to_run_as_attribute() {}

    #[run_as(component("singleton"))]
    //~^ error: Unexpected literal type `non-word`
    fn literal_entity_in_sub_attribute_passed_to_run_as_attribute() {}

    #[run_after]
    //~^ error: Unexpected meta-item format `word`
    fn wrong_run_after_attribute_syntax() {}

    #[run_after("action")]
    //~^ error: Unexpected meta-item format `literal`
    fn literal_passed_to_run_after_attribute() {}

    #[run_after_previous(TestAction)]
    //~^ error: Unexpected meta-item format `list`
    fn wrong_run_after_previous_attribute_syntax() {}

    #[run_after_previous_and]
    //~^ error: Unexpected meta-item format `word`
    fn wrong_run_after_previous_and_attribute_syntax() {}

    #[run_after_previous_and("action")]
    //~^ error: Unexpected meta-item format `literal`
    fn literal_passed_to_run_after_previous_and_attribute() {}
}
