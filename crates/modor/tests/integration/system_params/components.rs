use crate::system_params::{assert_iter, Text, Value};
use modor::{App, Built, EntityBuilder, Filter, Query, SingleMut, With};

struct QueryTester {
    done: bool,
}

#[singleton]
impl QueryTester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done: false })
    }

    #[run]
    fn run(&mut self, mut query: Query<'_, (&Value, Filter<With<Number>>)>) {
        assert_iter(query.iter().map(|v| v.0 .0), [1, 2, 3]);
        assert_iter(query.iter_mut().map(|v| v.0 .0), [1, 2, 3]);
        assert_iter(query.iter().rev().map(|v| v.0 .0), [3, 2, 1]);
        assert_iter(query.iter_mut().rev().map(|v| v.0 .0), [3, 2, 1]);
        assert_eq!(query.get(10).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(10).map(|v| v.0 .0), None);
        assert_eq!(query.get(5).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(5).map(|v| v.0 .0), None);
        assert_eq!(query.get(3).map(|v| v.0 .0), None);
        assert_eq!(query.get_mut(3).map(|v| v.0 .0), None);
        assert_eq!(query.get(4).map(|v| v.0 .0), Some(2));
        assert_eq!(query.get_mut(4).map(|v| v.0 .0), Some(2));
        let (left, right) = query.get_both_mut(4, 2);
        assert_eq!(left.map(|v| v.0 .0), Some(2));
        assert_eq!(right.map(|v| v.0 .0), Some(1));
        let (left, right) = query.get_both_mut(2, 4);
        assert_eq!(left.map(|v| v.0 .0), Some(1));
        assert_eq!(right.map(|v| v.0 .0), Some(2));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(150));
    }
}

struct StreamCollector(Vec<u32>);

#[singleton]
impl StreamCollector {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(vec![]))
    }
}

struct Number;

#[entity]
impl Number {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Value(value))
    }

    fn build_without_value() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    fn build_with_additional_component(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Value(value))
            .with(Text(String::from("other")))
    }

    #[run]
    fn collect(value: &Value, mut collector: SingleMut<'_, StreamCollector>) {
        collector.0.push(value.0);
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

struct OtherNumber;

#[entity]
impl OtherNumber {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Value(value))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn iterate_on_component_reference() {
    App::new()
        .with_entity(QueryTester::build())
        .with_entity(StreamCollector::build())
        .with_entity(Number::build(1))
        .with_entity(OtherNumber::build(10))
        .with_entity(Number::build(2))
        .with_entity(Number::build_without_value())
        .with_entity(Number::build_with_additional_component(3))
        .updated()
        .assert::<With<StreamCollector>>(1, |e| {
            e.has(|c: &StreamCollector| assert_eq!(c.0, [1, 2, 3]))
        })
        .assert::<With<QueryTester>>(1, |e| e.has(|c: &QueryTester| assert!(c.done)));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_must_use)]
fn run_systems_in_parallel() {
    modor_internal::retry!(10, {
        let start = instant::Instant::now();
        App::new()
            .with_thread_count(2)
            .with_entity(QueryTester::build())
            .with_entity(StreamCollector::build())
            .with_entity(Number::build(1))
            .with_entity(OtherNumber::build(10))
            .with_entity(Number::build(2))
            .with_entity(Number::build_without_value())
            .with_entity(Number::build_with_additional_component(3))
            .updated();
        assert!(start.elapsed() < std::time::Duration::from_millis(200));
    });
}
