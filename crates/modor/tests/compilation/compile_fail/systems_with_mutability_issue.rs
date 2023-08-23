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
    fn const_and_mut_same_component(_: &C1, _: &C3, _: &mut C1) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn mut_and_const_same_component(_: &C3, _: &mut C1, _: &C1) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn mut_and_mut_same_component(_: &mut C1, _: &mut C1, _: &C3) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn const_and_mut_same_option_component(_: Option<&C1>, _: &C3, _: Option<&mut C1>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn const_and_mut_option_same_component(_: &C1, _: &C3, _: Option<&mut C1>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn two_worlds(_: World<'_>, _: &C3, _: World<'_>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn world_and_entity_mut(_: World<'_>, _: EntityMut<'_>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn single_and_component(_: Single<'_, Singleton, &mut C1>, _: &mut C1) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn single_and_query(_: Single<'_, Singleton, &mut C1>, _: Query<'_, &mut C1>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn two_singles(_: Single<'_, Singleton, &mut C1>, _: Single<'_, Singleton, &mut C1>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn option_single_and_component(_: Option<Single<'_, Singleton, &mut C1>>, _: &mut C1) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn option_single_and_query(_: Option<Single<'_, Singleton, &mut C1>>, _: Query<'_, &mut C1>) {
        //~^ error: multiple applicable items in scope
        //~| is defined in an impl of the trait `modor::SystemWithParams`
        //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
    }

    #[run]
    fn two_option_singles(
        //~^ error: multiple applicable items in scope
        //~| is defined in an impl of the trait `modor::SystemWithParams`
        //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
        _: Option<Single<'_, Singleton, &mut C1>>,
        _: Option<Single<'_, Singleton, &mut C1>>,
    ) {
    }

    #[run]
    fn incompatible_tuples(_: (&C1,), _: &C3, _: (&mut C1,)) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn incompatible_queries(_: Query<'_, (&C1,)>, _: &C3, _: Query<'_, (&mut C1,)>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn tuple_with_incompatible_params(_: &C3, _: (&mut C1, &C1)) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn query_with_incompatible_params<'a>(_: &C3, _: Query<'a, (&'a mut C1, &'a C1)>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn nested_incompatible_params(_: (&C2, (&C4, (&mut C1,))), _: &C3, _: (&C2, (&C1,))) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`

    #[run]
    fn two_simple_queries(_: Query<'_, &mut C1>, _: Query<'_, &mut C1>) {}
    //~^ error: multiple applicable items in scope
    //~| is defined in an impl of the trait `modor::SystemWithParams`
    //~| is defined in an impl of the trait `modor::SystemWithParamMutabilityIssue`
}
