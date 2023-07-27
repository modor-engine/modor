use crate::system_params::{Value, ValueSingleton};
use modor::{App, BuiltEntity, EntityBuilder, Single, With};

#[modor_test]
fn run_system_with_param_for_present_singleton() {
    let singleton = EntityBuilder::new()
        .component(ValueSingleton(5))
        .component(Value(10));
    App::new()
        .with_entity(singleton)
        .with_entity(ValueSingletonUpdater::default())
        .with_entity(ValueSingletonUpdater::default())
        .updated()
        .assert::<With<ValueSingletonUpdater>>(2, |e| {
            e.has(|s: &ValueSingletonUpdater| assert_eq!(s.0, Some(Some(10))))
        })
        .assert::<With<ValueSingleton>>(1, |e| e.has(|s: &Value| assert_eq!(s.0, 12)));
}

#[modor_test]
fn run_system_with_param_for_missing_singleton() {
    App::new()
        .with_entity(ValueSingletonUpdater::default())
        .updated()
        .assert::<With<ValueSingletonUpdater>>(1, |e| {
            e.has(|s: &ValueSingletonUpdater| assert_eq!(s.0, Some(None)))
        });
}

#[modor_test]
fn run_system_with_param_for_not_matching_param() {
    App::new()
        .with_entity(ValueSingleton(5))
        .with_entity(ValueSingletonUpdater::default())
        .updated()
        .assert::<With<ValueSingletonUpdater>>(1, |e| {
            e.has(|s: &ValueSingletonUpdater| assert_eq!(s.0, Some(None)))
        });
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel_with_const_param() {
    modor_internal::retry!(
        60,
        assert!(are_systems_run_in_parallel!(
            (),
            Option<Single<'_, ValueSingleton, &ValueSingleton>>
        ))
    );
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel_with_mut_param() {
    assert!(!are_systems_run_in_parallel!(
        (),
        Option<Single<'_, ValueSingleton, &mut ValueSingleton>>
    ));
}

#[derive(Component, Default)]
struct ValueSingletonUpdater(Option<Option<u32>>);

#[systems]
impl ValueSingletonUpdater {
    #[run]
    fn retrieve_value(&mut self, singleton: Option<Single<'_, ValueSingleton, &Value>>) {
        self.0 = Some(singleton.map(|s| s.get().0));
    }

    #[run_after_previous]
    fn update_value(singleton: Option<Single<'_, ValueSingleton, &mut Value>>) {
        if let Some(mut singleton) = singleton {
            singleton.get_mut().0 += 1;
        }
    }
}
