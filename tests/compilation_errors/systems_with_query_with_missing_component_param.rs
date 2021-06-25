use modor::*;

fn main() {
    system!(query_with_only_not_component_entity_params);
    system!(tuple_with_query_with_only_not_component_entity_params);
    system!(query_with_query_with_only_not_component_entity_params);

    system_once!(query_with_only_not_component_entity_params);
    system_once!(tuple_with_query_with_only_not_component_entity_params);
    system_once!(query_with_query_with_only_not_component_entity_params);

    entity_system!(query_with_only_not_component_entity_params);
    entity_system!(tuple_with_query_with_only_not_component_entity_params);
    entity_system!(query_with_query_with_only_not_component_entity_params);
}

fn query_with_only_not_component_entity_params(
    _: Query<'_, (&u32,)>,
    _: Query<'_, (Option<&i64>,)>,
) {
}

fn tuple_with_query_with_only_not_component_entity_params(
    _: Query<'_, (&u32,)>,
    _: (Query<'_, (&u32,)>, Query<'_, (Option<&i64>,)>),
) {
}

fn query_with_query_with_only_not_component_entity_params<'a>(
    _: Query<'_, (&u32,)>,
    _: Query<'a, (Query<'a, (&'a u32,)>, Query<'a, (Option<&'a i64>,)>)>,
) {
}
