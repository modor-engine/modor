use crate::system_params::assert_iter;
use modor::{App, BuiltEntity, Entity, EntityBuilder, Filter, Query, With};

#[derive(SingletonComponent, Default)]
struct QueryTester {
    empty_done: bool,
    one_item_done: bool,
    two_item_done: bool,
    more_than_two_item_done: bool,
}

#[systems]
impl QueryTester {
    #[run]
    fn run_empty(&mut self, mut query: Query<'_, ((), Filter<With<Values>>)>) {
        assert_iter(query.iter().map(|v| v.0), [(), (), ()]);
        assert_iter(query.iter_mut().map(|v| v.0), [(), (), ()]);
        assert_iter(query.iter().rev().map(|v| v.0), [(), (), ()]);
        assert_iter(query.iter_mut().rev().map(|v| v.0), [(), (), ()]);
        assert_eq!(query.get(10).map(|v| v.0), None);
        assert_eq!(query.get_mut(10).map(|v| v.0), None);
        assert_eq!(query.get(1).map(|v| v.0), None);
        assert_eq!(query.get_mut(1).map(|v| v.0), None);
        assert_eq!(query.get(2).map(|v| v.0), Some(()));
        assert_eq!(query.get_mut(2).map(|v| v.0), Some(()));
        let (left, right) = query.get_both_mut(2, 3);
        assert_eq!(left.map(|v| v.0), Some(()));
        assert_eq!(right.map(|v| v.0), Some(()));
        self.empty_done = true;
    }

    #[run]
    fn run_one_item(&mut self, mut query: Query<'_, ((Entity<'_>,), Filter<With<Values>>)>) {
        assert_iter(query.iter().map(|v| v.0 .0.id()), [3, 2, 4]);
        assert_iter(query.iter_mut().map(|v| v.0 .0.id()), [3, 2, 4]);
        assert_iter(query.iter().rev().map(|v| v.0 .0.id()), [4, 2, 3]);
        assert_iter(query.iter_mut().rev().map(|v| v.0 .0.id()), [4, 2, 3]);
        assert_eq!(query.get(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(2).map(|v| v.0 .0.id()), Some(2));
        assert_eq!(query.get_mut(2).map(|v| v.0 .0.id()), Some(2));
        let (left, right) = query.get_both_mut(2, 3);
        assert_eq!(left.map(|v| v.0 .0.id()), Some(2));
        assert_eq!(right.map(|v| v.0 .0.id()), Some(3));
        self.one_item_done = true;
    }

    #[allow(clippy::type_complexity)]
    #[run]
    fn run_two_items(
        &mut self,
        mut query: Query<'_, ((Entity<'_>, &Value1), Filter<With<Values>>)>,
    ) {
        assert_iter(query.iter().map(|v| v.0 .0.id()), [3, 2]);
        assert_iter(query.iter_mut().map(|v| v.0 .0.id()), [3, 2]);
        assert_iter(query.iter().rev().map(|v| v.0 .0.id()), [2, 3]);
        assert_iter(query.iter_mut().rev().map(|v| v.0 .0.id()), [2, 3]);
        assert_eq!(query.get(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(4).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(4).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(2).map(|v| v.0 .0.id()), Some(2));
        assert_eq!(query.get_mut(2).map(|v| v.0 .0.id()), Some(2));
        let (left, right) = query.get_both_mut(2, 3);
        assert_eq!(left.map(|v| v.0 .0.id()), Some(2));
        assert_eq!(right.map(|v| v.0 .0.id()), Some(3));
        self.two_item_done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[allow(clippy::type_complexity)]
    #[run]
    fn run_more_than_two_items(
        &mut self,
        mut query: Query<'_, ((Entity<'_>, &Value1, &Value2), Filter<With<Values>>)>,
    ) {
        assert_iter(query.iter().map(|v| v.0 .0.id()), [2]);
        assert_iter(query.iter_mut().map(|v| v.0 .0.id()), [2]);
        assert_iter(query.iter().rev().map(|v| v.0 .0.id()), [2]);
        assert_iter(query.iter_mut().rev().map(|v| v.0 .0.id()), [2]);
        assert_eq!(query.get(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(1).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(4).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get_mut(4).map(|v| v.0 .0.id()), None);
        assert_eq!(query.get(2).map(|v| v.0 .0.id()), Some(2));
        assert_eq!(query.get_mut(2).map(|v| v.0 .0.id()), Some(2));
        let (left, right) = query.get_both_mut(2, 3);
        assert_eq!(left.map(|v| v.0 .0.id()), Some(2));
        assert_eq!(right.map(|v| v.0 .0.id()), None);
        self.more_than_two_item_done = true;
    }
}

#[derive(Component, NoSystem)]
struct Value1(u32);

#[derive(Component, NoSystem)]
struct Value2(u32);

#[derive(Component)]
struct Values {
    value1: bool,
    value2: bool,
    empty_done: bool,
    one_item_done: bool,
    two_item_done: bool,
    more_than_two_item_done: bool,
}

#[systems]
impl Values {
    fn build(value1: bool, value2: bool) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self {
                value1,
                value2,
                empty_done: false,
                one_item_done: false,
                two_item_done: false,
                more_than_two_item_done: false,
            })
            .with_option(value1.then_some(Value1(10)))
            .with_option(value2.then_some(Value2(20)))
    }

    #[run]
    fn iterate_on_empty_tuple(&mut self, _: ()) {
        self.empty_done = true;
    }

    #[run]
    fn iterate_on_one_item_tuple(&mut self, (value1,): (&Value1,)) {
        assert_eq!(value1.0, 10);
        self.one_item_done = true;
    }

    #[run]
    fn iterate_on_two_items_tuple((self_, value1): (&mut Self, &Value1)) {
        assert_eq!(value1.0, 10);
        self_.two_item_done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }

    #[run]
    fn iterate_on_more_than_two_items_tuple(
        (self_, value1, value2): (&mut Self, &Value1, &mut Value2),
    ) {
        assert_eq!(value1.0, 10);
        assert_eq!(value2.0, 20);
        self_.more_than_two_item_done = true;
    }
}

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(QueryTester::default())
        .with_child(Values::build(true, true))
        .with_child(Values::build(true, false))
        .with_child(Values::build(false, true))
}

#[modor_test]
fn iteration_on_tuple() {
    App::new()
        .with_entity(entities())
        .updated()
        .assert::<With<QueryTester>>(1, |e| {
            e.has(|q: &QueryTester| {
                assert!(q.empty_done);
                assert!(q.one_item_done);
                assert!(q.two_item_done);
                assert!(q.more_than_two_item_done);
            })
        })
        .assert::<With<Values>>(3, |e| {
            e.has(|v: &Values| {
                assert!(v.empty_done);
                if v.value1 && v.value2 {
                    assert!(v.one_item_done);
                    assert!(v.two_item_done);
                    assert!(v.more_than_two_item_done);
                } else if v.value1 && !v.value2 {
                    assert!(v.one_item_done);
                    assert!(v.two_item_done);
                    assert!(!v.more_than_two_item_done);
                } else {
                    assert!(!v.one_item_done);
                    assert!(!v.two_item_done);
                    assert!(!v.more_than_two_item_done);
                }
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
        assert!(start.elapsed() < std::time::Duration::from_millis(200));
    });
}
