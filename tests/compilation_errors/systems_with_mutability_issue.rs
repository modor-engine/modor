use modor::*;

fn main() {
    system!(const_and_mut_same_component);
    system!(mut_and_const_same_component);
    system!(mut_and_mut_same_component);
    system!(const_and_mut_same_option_component);
    system!(two_worlds);
    system!(incompatible_tuples);
    system!(incompatible_queries);
    system!(tuple_with_incompatible_params);
    system!(query_with_incompatible_params);
    system!(nested_incompatible_params);
}

fn const_and_mut_same_component(_: &u32, _: &String, _: &mut u32) {}

fn mut_and_const_same_component(_: &String, _: &mut u32, _: &u32) {}

fn mut_and_mut_same_component(_: &mut u32, _: &mut u32, _: &String) {}

fn const_and_mut_same_option_component(_: Option<&u32>, _: &String, _: Option<&mut u32>) {}

fn two_worlds(_: World<'_>, _: &String, _: World<'_>) {}

fn incompatible_tuples(_: (&u32,), _: &String, _: (&mut u32,)) {}

fn incompatible_queries(_: Query<'_, (&u32,)>, _: &String, _: Query<'_, (&mut u32,)>) {}

fn tuple_with_incompatible_params(_: &String, _: (&mut u32, &u32)) {}

fn query_with_incompatible_params<'a>(_: &String, _: Query<'a, (&'a mut u32, &'a u32)>) {}

fn nested_incompatible_params(_: (&i64, (&u64, (&mut u32,))), _: &String, _: (&i64, (&u32,))) {}
