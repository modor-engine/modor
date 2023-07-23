use crate::system_params::ValueSingleton;
use modor::{App, Single, With};

#[modor_test]
fn run_system_with_param_for_present_singleton() {
    App::new()
        .with_entity(ValueSingleton(10))
        .with_entity(ValueSingletonState::default())
        .updated()
        .assert::<With<ValueSingletonState>>(1, |e| {
            e.has(|s: &ValueSingletonState| assert_eq!(s.id, Some(0)))
                .has(|s: &ValueSingletonState| assert_eq!(s.value, Some(10)))
        });
}

#[modor_test]
fn run_system_with_param_for_missing_singleton() {
    App::new()
        .with_entity(ValueSingletonState::default())
        .updated()
        .assert::<With<ValueSingletonState>>(1, |e| {
            e.has(|s: &ValueSingletonState| assert_eq!(s.id, None))
                .has(|s: &ValueSingletonState| assert_eq!(s.value, None))
        });
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    modor_internal::retry!(
        60,
        assert!(are_systems_run_in_parallel!((), Single<'_, ValueSingleton>))
    );
}

#[derive(SingletonComponent, Default)]
struct ValueSingletonState {
    id: Option<usize>,
    value: Option<u32>,
}

#[systems]
impl ValueSingletonState {
    #[run]
    fn update(&mut self, value: Single<'_, ValueSingleton>) {
        self.id = Some(value.entity().id());
        self.value = Some(value.0);
    }
}
