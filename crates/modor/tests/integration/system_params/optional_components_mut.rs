use crate::system_params::{assert_iter, Number, OptionalValue, OtherNumber, Value};
use modor::{App, BuiltEntity, EntityBuilder, Filter, Query, SingleMut, With};

#[derive(SingletonComponent, Default)]
struct QueryTester {
    done: bool,
    done_complex: bool,
}

#[systems]
impl QueryTester {
    #[run]
    fn collect(&mut self, mut query: Query<'_, (Option<&mut Value>, Filter<With<Number>>)>) {
        let values = [Some(1), Some(2), None, Some(3)];
        assert_iter(query.iter().map(|v| v.0.map(|v| v.0)), values);
        assert_iter(query.iter_mut().map(|v| v.0.map(|v| v.0)), values);
        let rev_values = [Some(3), None, Some(2), Some(1)];
        assert_iter(query.iter().rev().map(|v| v.0.map(|v| v.0)), rev_values);
        assert_iter(query.iter_mut().rev().map(|v| v.0.map(|v| v.0)), rev_values);
        assert_eq!(query.get(10).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get_mut(10).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get(6).map(|v| v.0.map(|v| v.0)), Some(None));
        assert_eq!(query.get_mut(6).map(|v| v.0.map(|v| v.0)), Some(None));
        assert_eq!(query.get(4).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get_mut(4).map(|v| v.0.map(|v| v.0)), None);
        assert_eq!(query.get(5).map(|v| v.0.map(|v| v.0)), Some(Some(2)));
        assert_eq!(query.get_mut(5).map(|v| v.0.map(|v| v.0)), Some(Some(2)));
        let (left, right) = query.get_both_mut(5, 3);
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

#[derive(SingletonComponent, NoSystem, Default)]
struct Numbers(Vec<Option<u32>>);

#[derive(Component)]
struct RegisteredNumber;

#[systems]
impl RegisteredNumber {
    #[run]
    fn collect(value: Option<&mut Value>, mut collector: SingleMut<'_, Numbers>) {
        collector.0.push(value.map(|v| v.0));
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(QueryTester::default())
        .with_child(Numbers::default())
        .with_child(
            Number::build(1)
                .with(RegisteredNumber)
                .with(OptionalValue(1)),
        )
        .with_child(OtherNumber::build(10))
        .with_child(
            Number::build(2)
                .with(RegisteredNumber)
                .with(OptionalValue(2)),
        )
        .with_child(Number::build_without_value().with(RegisteredNumber))
        .with_child(Number::build_with_additional_component(3).with(RegisteredNumber))
}

#[modor_test]
fn iterate_on_component_reference() {
    App::new()
        .with_entity(entities())
        .updated()
        .assert::<With<Numbers>>(1, |e| {
            e.has(|c: &Numbers| assert_eq!(c.0, [Some(1), Some(2), None, Some(3)]))
        })
        .assert::<With<QueryTester>>(1, |e| {
            e.has(|c: &QueryTester| {
                assert!(c.done);
                assert!(c.done_complex);
            })
        });
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(entities())
        .updated();
    assert!(start.elapsed() > std::time::Duration::from_millis(400));
}
