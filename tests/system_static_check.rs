#![allow(clippy::trivially_copy_pass_by_ref)]
use modor::*;
use trybuild::TestCases;

fn no_param() {}

fn mandatory_component_param(_: &u32) {}

fn mandatory_component_in_tuple(_: Option<&u32>, _: Group<'_>, _: (&mut i64,)) {}

fn mandatory_component_in_sub_tuple(_: Option<&u32>, _: Group<'_>, _: (Entity<'_>, (&mut i64,))) {}

fn mandatory_component_in_query(_: Query<'_, (&mut i64,)>) {}

fn mandatory_component_in_sub_tuple_in_query<'a>(
    _: Query<'a, (Group<'a>, (Entity<'a>, (&'a mut i64,)))>,
) {
}

fn mandatory_component_in_query_in_query<'a>(_: Query<'a, (Query<'a, (&'a mut i64,)>,)>) {}

fn same_const_component(_: &u32, _: &u32) {}

fn different_mut_components(_: &mut u32, _: &mut i64) {}

fn all_entity_param_types(
    _: &mut u32,
    _: Option<&mut i64>,
    _: &i16,
    _: Option<&i16>,
    _: Entity<'_>,
    _: Group<'_>,
) {
}

fn entity_params_with_query(_: &mut u32, _: Group<'_>, _: Query<'_, (&mut i64,)>) {}

fn no_entity_part_without_mandatory_component_param(_: Group<'_>) {}

#[test]
fn create_valid_systems() {
    system!(no_param);
    system!(mandatory_component_param);
    system!(mandatory_component_in_tuple);
    system!(mandatory_component_in_sub_tuple);
    system!(mandatory_component_in_query);
    system!(mandatory_component_in_sub_tuple_in_query);
    system!(mandatory_component_in_query_in_query);
    system!(same_const_component);
    system!(different_mut_components);
    system!(all_entity_param_types);
    system!(entity_params_with_query);

    system_once!(no_param);
    system_once!(mandatory_component_param);
    system_once!(mandatory_component_in_tuple);
    system_once!(mandatory_component_in_sub_tuple);
    system_once!(mandatory_component_in_query);
    system_once!(mandatory_component_in_sub_tuple_in_query);
    system_once!(mandatory_component_in_query_in_query);
    system_once!(same_const_component);
    system_once!(different_mut_components);
    system_once!(all_entity_param_types);
    system_once!(entity_params_with_query);

    entity_system!(no_param);
    entity_system!(mandatory_component_param);
    entity_system!(mandatory_component_in_tuple);
    entity_system!(mandatory_component_in_sub_tuple);
    entity_system!(mandatory_component_in_query);
    entity_system!(mandatory_component_in_sub_tuple_in_query);
    entity_system!(mandatory_component_in_query_in_query);
    entity_system!(same_const_component);
    entity_system!(different_mut_components);
    entity_system!(all_entity_param_types);
    entity_system!(entity_params_with_query);
    entity_system!(no_entity_part_without_mandatory_component_param);
}

#[test]
fn create_invalid_systems() {
    let t = TestCases::new();
    t.compile_fail("tests/compilation_errors/*.rs");
}
