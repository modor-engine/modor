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
    fn collect(&mut self, mut query: Query<'_, (Option<&Value>, Filter<With<Number>>)>) {
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
        mut query: Query<'_, (&Value, Option<&OptionalValue>, Filter<With<Number>>)>,
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
    fn collect(value: Option<&Value>, mut collector: SingleMut<'_, Numbers>) {
        collector.0.push(value.map(|v| v.0));
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(QueryTester::default())
        .child_component(Numbers::default())
        .child_entity(
            Number::build(1)
                .component(RegisteredNumber)
                .component(OptionalValue(1)),
        )
        .child_entity(OtherNumber::build(10))
        .child_entity(
            Number::build(2)
                .component(RegisteredNumber)
                .component(OptionalValue(2)),
        )
        .child_entity(Number::build_without_value().component(RegisteredNumber))
        .child_entity(Number::build_with_additional_component(3).component(RegisteredNumber))
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
    modor_internal::retry!(60, {
        let start = instant::Instant::now();
        App::new()
            .with_thread_count(2)
            .with_entity(entities())
            .updated();
        assert!(start.elapsed() < std::time::Duration::from_millis(400));
    });
}
