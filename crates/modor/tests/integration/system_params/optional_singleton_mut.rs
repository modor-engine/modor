use crate::system_params::ValueSingleton;
use modor::{App, SingleMut, With};

#[modor_test]
fn run_system_with_param_for_present_singleton() {
    App::new()
        .with_entity(ValueSingleton(10))
        .with_entity(ValueSingletonState::default())
        .updated()
        .assert::<With<ValueSingletonState>>(1, |e| {
            e.has(|s: &ValueSingletonState| assert_eq!(s.id, Some(Some(0))))
                .has(|s: &ValueSingletonState| assert_eq!(s.value, Some(Some(10))))
        });
}

#[modor_test]
fn run_system_with_param_for_missing_singleton() {
    App::new()
        .with_entity(ValueSingletonState::default())
        .updated()
        .assert::<With<ValueSingletonState>>(1, |e| {
            e.has(|s: &ValueSingletonState| assert_eq!(s.id, Some(None)))
                .has(|s: &ValueSingletonState| assert_eq!(s.value, Some(None)))
        });
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    assert!(!are_systems_run_in_parallel!(
        (),
        Option<SingleMut<'_, ValueSingleton>>
    ));
}

#[derive(SingletonComponent, Default)]
struct ValueSingletonState {
    id: Option<Option<usize>>,
    value: Option<Option<u32>>,
}

#[systems]
impl ValueSingletonState {
    #[run]
    fn update(&mut self, value: Option<SingleMut<'_, ValueSingleton>>) {
        self.id = Some(value.as_ref().map(|v| v.entity().id()));
        self.value = Some(value.as_ref().map(|v| v.0));
    }
}
