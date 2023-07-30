use modor::{App, Component, NoSystem, Query, QuerySystemParam, SingleMut, With};
use std::marker::PhantomData;

// TODO: update doc

#[derive(Component, NoSystem)]
struct Value1;

#[derive(Component, NoSystem)]
struct Value2;

#[derive(SingletonComponent, NoSystem)]
struct IsSystemRun(bool);

#[derive(Component)]
struct GenericQueryTester<P>(PhantomData<P>)
where
    P: 'static + Sync + Send + QuerySystemParam;

#[systems]
impl<P> GenericQueryTester<P>
where
    P: 'static + Sync + Send + QuerySystemParam,
{
    #[run]
    fn update(_: Query<'_, P>, mut is_system_run: SingleMut<'_, '_, IsSystemRun>) {
        is_system_run.get_mut().0 = true;
    }
}

#[derive(Component)]
struct GenericComponentParamsTester<P1, P2>(PhantomData<(P1, P2)>)
where
    P1: Component,
    P2: Component;

#[systems]
impl<P1, P2> GenericComponentParamsTester<P1, P2>
where
    P1: Component,
    P2: Component,
{
    #[run]
    fn update(_: &mut P1, _: &mut P2) {}
}

#[test]
fn register_system_without_mutability_issue() {
    App::new()
        .with_entity(IsSystemRun(false))
        .with_entity(GenericQueryTester::<(&Value1, &Value1)>(PhantomData))
        .updated()
        .assert::<With<IsSystemRun>>(1, |e| e.has(|r: &IsSystemRun| assert!(r.0)));
}

#[test]
fn register_system_with_mutability_issue_caused_by_2_mut_components() {
    App::new()
        .with_entity(IsSystemRun(false))
        .with_entity(GenericQueryTester::<(&mut Value1, &mut Value1)>(
            PhantomData,
        ))
        .updated()
        .assert::<With<IsSystemRun>>(1, |e| e.has(|r: &IsSystemRun| assert!(!r.0)));
    App::new()
        .with_entity(IsSystemRun(false))
        .with_entity(GenericComponentParamsTester::<Value1, Value1>(PhantomData))
        .updated()
        .assert::<With<IsSystemRun>>(1, |e| e.has(|r: &IsSystemRun| assert!(!r.0)));
}

#[test]
fn register_system_with_mutability_issue_caused_by_1_mut_1_const_components() {
    App::new()
        .with_entity(IsSystemRun(false))
        .with_entity(GenericQueryTester::<(&mut Value1, &Value1)>(PhantomData))
        .updated()
        .assert::<With<IsSystemRun>>(1, |e| e.has(|r: &IsSystemRun| assert!(!r.0)));
}
