use modor::*;

fn main() {
    system!(only_option_component);
    system!(only_group);
    system!(multiple_not_component_entity_params);
    system!(not_component_entity_params_and_query);
    system!(tuple_with_not_component_entity_params);

    system_once!(only_option_component);
    system_once!(only_group);
    system_once!(multiple_not_component_entity_params);
    system_once!(not_component_entity_params_and_query);
    system_once!(tuple_with_not_component_entity_params);
}

fn only_option_component(_: Option<&u32>) {}

fn only_group(_: Group<'_>) {}

fn multiple_not_component_entity_params(_: Option<&u32>, _: Group<'_>) {}

fn not_component_entity_params_and_query(_: Option<&u32>, _: Group<'_>, _: Query<'_, (&u32,)>) {}

fn tuple_with_not_component_entity_params(
    _: ((Entity<'_>, (Option<&u32>,)), Option<&i64>, (Group<'_>,)),
) {
}
