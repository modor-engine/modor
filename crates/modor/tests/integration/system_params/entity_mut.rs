use crate::system_params::{OtherValue, Value};
use modor::{App, EntityMut, With};

#[modor_test]
fn access_entity() {
    Tester::run(|e| assert_eq!(e.entity().depth(), 0));
}

#[modor_test]
fn access_world() {
    Tester::run(|e| e.world().create_root_entity(Value(1))).assert::<With<Value>>(1, |e| e);
}

#[modor_test]
fn create_child() {
    Tester::run(|e| e.create_child(OtherValue(1)))
        .assert::<With<OtherValue>>(1, |e| e.has_parent::<With<Tester>>());
}

#[modor_test]
fn delete() {
    Tester::run(|e| e.delete()).assert::<With<Tester>>(0, |e| e);
}

#[modor_test]
fn add_component() {
    Tester::run(|e| e.add_component(Value(2)))
        .assert::<With<Tester>>(1, |e| e.has(|c: &Value| assert_eq!(c.0, 2)));
}

#[modor_test]
fn delete_component() {
    Tester::run(|e| e.delete_component::<Tester>()).assert::<()>(1, |e| e.has_not::<Tester>());
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    assert!(!are_systems_run_in_parallel!((), EntityMut<'_>));
}

#[derive(SingletonComponent)]
struct Tester {
    test_fn: fn(&mut EntityMut<'_>),
    is_done: bool,
}

#[systems]
impl Tester {
    fn run(test_fn: fn(&mut EntityMut<'_>)) -> App {
        App::new()
            .with_entity(Self {
                test_fn,
                is_done: false,
            })
            .updated()
            .assert::<With<Self>>(.., |e| e.has(|t: &Self| assert!(t.is_done)))
    }

    #[run]
    #[allow(clippy::redundant_closure_call)]
    fn update(&mut self, mut entity: EntityMut<'_>) {
        (self.test_fn)(&mut entity);
        self.is_done = true;
    }
}
