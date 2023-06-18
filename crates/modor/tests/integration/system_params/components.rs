use crate::system_params::{assert_iter, Number, OtherNumber, Value};
use modor::{App, BuiltEntity, EntityBuilder, Filter, Query, SingleMut, With};

#[derive(SingletonComponent, Default)]
struct QueryTester {
    done: bool,
}

#[systems]
impl QueryTester {
    #[run]
    fn run(&mut self, mut query: Query<'_, (&Value, Filter<With<Number>>)>) {
        assert_iter(query.iter().map(|v| v.0 .0), [1, 2, 3]);
        assert_iter(query.iter_mut().map(|v| v.0 .0), [1, 2, 3]);
        assert_iter(query.iter().rev().map(|v| v.0 .0), [3, 2, 1]);
        assert_iter(query.iter_mut().rev().map(|v| v.0 .0), [3, 2, 1]);
        assert_eq!(query.get(10).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(10).map(|v| v.0 .0), None);
        assert_eq!(query.get(6).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(6).map(|v| v.0 .0), None);
        assert_eq!(query.get(4).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(4).map(|v| v.0 .0), None);
        assert_eq!(query.get(5).map(|v| v.0 .0), Some(2));
        assert_eq!(query.get_mut(5).map(|v| v.0 .0), Some(2));
        let (left, right) = query.get_both_mut(5, 3);
        assert_eq!(left.map(|v| v.0 .0), Some(2));
        assert_eq!(right.map(|v| v.0 .0), Some(1));
        let (left, right) = query.get_both_mut(3, 5);
        assert_eq!(left.map(|v| v.0 .0), Some(1));
        assert_eq!(right.map(|v| v.0 .0), Some(2));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(150));
    }
}

#[derive(SingletonComponent, NoSystem, Default)]
struct Numbers(Vec<u32>);

#[derive(Component)]
struct RegisteredNumber;

#[systems]
impl RegisteredNumber {
    #[run]
    fn collect(value: &Value, mut numbers: SingleMut<'_, Numbers>) {
        numbers.0.push(value.0);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(QueryTester::default())
        .with_child(Numbers::default())
        .with_child(Number::build(1).with(RegisteredNumber))
        .with_child(OtherNumber::build(10))
        .with_child(Number::build(2).with(RegisteredNumber))
        .with_child(Number::build_without_value().with(RegisteredNumber))
        .with_child(Number::build_with_additional_component(3).with(RegisteredNumber))
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn iterate_on_component_reference() {
    App::new()
        .with_entity(entities())
        .updated()
        .assert::<With<Numbers>>(1, |e| e.has(|c: &Numbers| assert_eq!(c.0, [1, 2, 3])))
        .assert::<With<QueryTester>>(1, |e| e.has(|c: &QueryTester| assert!(c.done)));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_systems_in_parallel() {
    modor_internal::retry!(60, {
        let start = instant::Instant::now();
        App::new()
            .with_thread_count(2)
            .with_entity(entities())
            .updated();
        assert!(start.elapsed() < std::time::Duration::from_millis(300));
    });
}
