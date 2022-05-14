#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

struct SingletonEntity;

#[singleton]
impl SingletonEntity {}

struct EntityWithInvalidSystems;

#[entity]
impl EntityWithInvalidSystems {
    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_component(_: &u32, _: &String, _: &mut u32) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_component(_: &String, _: &mut u32, _: &u32) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_component(_: &mut u32, _: &mut u32, _: &String) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_component(_: Option<&u32>, _: &String, _: Option<&mut u32>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_component(_: &u32, _: &String, _: Option<&mut u32>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_singleton(
        _: Single<'_, SingletonEntity>,
        _: SingleMut<'_, SingletonEntity>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_singleton_and_component(
        _: Single<'_, SingletonEntity>,
        _: &mut SingletonEntity,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_singleton(
        _: SingleMut<'_, SingletonEntity>,
        _: Single<'_, SingletonEntity>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_singleton_and_component(
        _: SingleMut<'_, SingletonEntity>,
        _: &SingletonEntity,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_singleton(
        _: SingleMut<'_, SingletonEntity>,
        _: SingleMut<'_, SingletonEntity>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_singleton_and_component(
        _: SingleMut<'_, SingletonEntity>,
        _: &mut SingletonEntity,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_singleton(
        _: Option<Single<'_, SingletonEntity>>,
        _: Option<SingleMut<'_, SingletonEntity>>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_singleton_and_component(
        _: Option<Single<'_, SingletonEntity>>,
        _: Option<&mut SingletonEntity>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_singleton(
        _: Single<'_, SingletonEntity>,
        _: &String,
        _: Option<SingleMut<'_, SingletonEntity>>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_singleton_and_component(
        _: Single<'_, SingletonEntity>,
        _: &String,
        _: Option<&mut SingletonEntity>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn two_worlds(_: World<'_>, _: &String, _: World<'_>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn incompatible_tuples(_: (&u32,), _: &String, _: (&mut u32,)) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn incompatible_queries(_: Query<'_, (&u32,)>, _: &String, _: Query<'_, (&mut u32,)>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn tuple_with_incompatible_params(_: &String, _: (&mut u32, &u32)) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn query_with_incompatible_params<'a>(_: &String, _: Query<'a, (&'a mut u32, &'a u32)>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn nested_incompatible_params(_: (&i64, (&u64, (&mut u32,))), _: &String, _: (&i64, (&u32,))) {}
}
