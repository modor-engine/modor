#[macro_use]
extern crate modor;

use modor::*;

fn main() {}

#[derive(Component, NoSystem)]
struct C1;

#[derive(Component, NoSystem)]
struct C2;

#[derive(Component, NoSystem)]
struct C3;

#[derive(Component, NoSystem)]
struct C4;

#[derive(SingletonComponent, NoSystem)]
struct Singleton;

#[derive(Component)]
struct ComponentWithInvalidSystems;

#[systems]
impl ComponentWithInvalidSystems {
    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_component(_: &C1, _: &C3, _: &mut C1) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_component(_: &C3, _: &mut C1, _: &C1) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_component(_: &mut C1, _: &mut C1, _: &C3) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_component(_: Option<&C1>, _: &C3, _: Option<&mut C1>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_component(_: &C1, _: &C3, _: Option<&mut C1>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_singleton(_: Single<'_, Singleton>, _: SingleMut<'_, Singleton>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_singleton_and_component(_: Single<'_, Singleton>, _: &mut Singleton) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_singleton(_: SingleMut<'_, Singleton>, _: Single<'_, Singleton>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_const_same_singleton_and_component(_: SingleMut<'_, Singleton>, _: &Singleton) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_singleton(_: SingleMut<'_, Singleton>, _: SingleMut<'_, Singleton>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn mut_and_mut_same_singleton_and_component(_: SingleMut<'_, Singleton>, _: &mut Singleton) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_singleton(
        _: Option<Single<'_, Singleton>>,
        _: Option<SingleMut<'_, Singleton>>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_same_option_singleton_and_component(
        _: Option<Single<'_, Singleton>>,
        _: Option<&mut Singleton>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_singleton(
        _: Single<'_, Singleton>,
        _: &C3,
        _: Option<SingleMut<'_, Singleton>>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn const_and_mut_option_same_singleton_and_component(
        _: Single<'_, Singleton>,
        _: &C3,
        _: Option<&mut Singleton>,
    ) {
    }

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn two_worlds(_: World<'_>, _: &C3, _: World<'_>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn world_and_entity_mut(_: World<'_>, _: EntityMut<'_>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn incompatible_tuples(_: (&C1,), _: &C3, _: (&mut C1,)) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn incompatible_queries(_: Query<'_, (&C1,)>, _: &C3, _: Query<'_, (&mut C1,)>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn tuple_with_incompatible_params(_: &C3, _: (&mut C1, &C1)) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn query_with_incompatible_params<'a>(_: &C3, _: Query<'a, (&'a mut C1, &'a C1)>) {}

    #[run]
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    fn nested_incompatible_params(_: (&C2, (&C4, (&mut C1,))), _: &C3, _: (&C2, (&C1,))) {}
}
