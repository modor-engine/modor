use crate::system_params::{assert_iter, OptionalValue, Text, Value};
use modor::{App, Built, EntityBuilder, Filter, Query, SingleMut, With};

struct QueryTester {
    done: bool,
    done_complex: bool,
}

#[singleton]
impl QueryTester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            done: false,
            done_complex: false,
        })
    }

    #[run]
    fn collect(&mut self, mut query: Query<'_, (Option<&mut Value>, Filter<With<Number>>)>) {
        let values = [None, Some(1), Some(2), Some(3)];
        assert_iter(query.iter().map(|v| v.0.map(|v| v.0)), values);
        assert_iter(query.iter_mut().map(|v| v.0.map(|v| v.0)), values);
        let rev_values = [Some(3), Some(2), Some(1), None];
        assert_iter(query.iter().rev().map(|v| v.0.map(|v| v.0)), rev_values);
        assert_iter(query.iter_mut().rev().map(|v| v.0.map(|v| v.0)), rev_values);
        assert_eq!(query.get(10).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get_mut(10).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get(5).map(|v| v.0.map(|v| v.0)), Some(None));
        assert_eq!(query.get_mut(5).map(|v| v.0.map(|v| v.0)), Some(None));
        assert_eq!(query.get(3).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get_mut(3).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get(4).map(|v| v.0.map(|v| v.0)), Some(Some(2)));
        assert_eq!(query.get_mut(4).map(|v| v.0.map(|v| v.0)), Some(Some(2)));
        let (left, right) = query.get_both_mut(4, 2);
        assert_eq!(left.map(|v| v.0.map(|v| v.0)), Some(Some(2)));
        assert_eq!(right.map(|v| v.0.map(|v| v.0)), Some(Some(1)));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(200));
    }

    #[allow(clippy::type_complexity)]
    #[run]
    fn collect_complex(
        &mut self,
        mut query: Query<'_, (&Value, Option<&mut OptionalValue>, Filter<With<Number>>)>,
    ) {
        let values = [Some(1), Some(2), None];
        assert_iter(query.iter().map(|v| v.1.cloned().map(|v| v.0)), values);
        assert_iter(query.iter_mut().map(|v| v.1.cloned().map(|v| v.0)), values);
        let rev_values = [None, Some(2), Some(1)];
        assert_iter(
            query.iter().rev().map(|v| v.1.cloned().map(|v| v.0)),
            rev_values,
        );
        assert_iter(
            query.iter_mut().rev().map(|v| v.1.cloned().map(|v| v.0)),
            rev_values,
        );
        self.done_complex = true;
    }
}

struct StreamCollector(Vec<Option<u32>>);

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
        EntityBuilder::new(Self)
            .with(Value(value))
            .with(OptionalValue(value))
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
    fn collect(value: Option<&mut Value>, mut collector: SingleMut<'_, StreamCollector>) {
        collector.0.push(value.map(|v| v.0));
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
            e.has(|c: &StreamCollector| assert_eq!(c.0, [None, Some(1), Some(2), Some(3)]))
        })
        .assert::<With<QueryTester>>(1, |e| {
            e.has(|c: &QueryTester| {
                assert!(c.done);
                assert!(c.done_complex);
            })
        });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_systems_in_parallel() {
    let start = instant::Instant::now();
    let _app = App::new()
        .with_thread_count(2)
        .with_entity(QueryTester::build())
        .with_entity(StreamCollector::build())
        .with_entity(Number::build(1))
        .with_entity(OtherNumber::build(10))
        .with_entity(Number::build(2))
        .with_entity(Number::build_without_value())
        .with_entity(Number::build_with_additional_component(3))
        .updated();
    assert!(start.elapsed() > std::time::Duration::from_millis(400));
}
