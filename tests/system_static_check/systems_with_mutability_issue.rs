extern crate modor;

use modor::*;

fn main() {
    system!(const_and_mut_same_component);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(mut_and_const_same_component);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(mut_and_mut_same_component);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(const_and_mut_same_option_component);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(const_and_mut_option_same_component);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(const_and_mut_same_global);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(mut_and_const_same_global);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(mut_and_mut_same_global);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(const_and_mut_same_option_global);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(const_and_mut_option_same_global);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(two_worlds);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(incompatible_tuples);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(incompatible_queries);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(tuple_with_incompatible_params);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(query_with_incompatible_params);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    system!(nested_incompatible_params);
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
}

struct TestGlobal {}

impl Global for TestGlobal {}

fn const_and_mut_same_component(_: &u32, _: &String, _: &mut u32) {}

fn mut_and_const_same_component(_: &String, _: &mut u32, _: &u32) {}

fn mut_and_mut_same_component(_: &mut u32, _: &mut u32, _: &String) {}

fn const_and_mut_same_option_component(_: Option<&u32>, _: &String, _: Option<&mut u32>) {}

fn const_and_mut_option_same_component(_: &u32, _: &String, _: Option<&mut u32>) {}

fn const_and_mut_same_global(_: Glob<'_, TestGlobal>, _: GlobMut<'_, TestGlobal>) {}

fn mut_and_const_same_global(_: GlobMut<'_, TestGlobal>, _: Glob<'_, TestGlobal>) {}

fn mut_and_mut_same_global(_: GlobMut<'_, TestGlobal>, _: GlobMut<'_, TestGlobal>) {}

fn const_and_mut_same_option_global(
    _: Option<Glob<'_, TestGlobal>>,
    _: Option<GlobMut<'_, TestGlobal>>,
) {
}

fn const_and_mut_option_same_global(
    _: Glob<'_, TestGlobal>,
    _: &String,
    _: Option<GlobMut<'_, TestGlobal>>,
) {
}

fn two_worlds(_: World<'_>, _: &String, _: World<'_>) {}

fn incompatible_tuples(_: (&u32,), _: &String, _: (&mut u32,)) {}

fn incompatible_queries(_: Query<'_, (&u32,)>, _: &String, _: Query<'_, (&mut u32,)>) {}

fn tuple_with_incompatible_params(_: &String, _: (&mut u32, &u32)) {}

fn query_with_incompatible_params<'a>(_: &String, _: Query<'a, (&'a mut u32, &'a u32)>) {}

fn nested_incompatible_params(_: (&i64, (&u64, (&mut u32,))), _: &String, _: (&i64, (&u32,))) {}
