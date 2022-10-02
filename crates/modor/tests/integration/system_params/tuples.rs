use crate::system_params::assert_iter;
use modor::testing::TestApp;
use modor::{App, Built, Entity, EntityBuilder, Query, With};

struct QueryTester {
    empty_done: bool,
    one_item_done: bool,
    two_item_done: bool,
    more_than_two_item_done: bool,
}

#[singleton]
impl QueryTester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            empty_done: false,
            one_item_done: false,
            two_item_done: false,
            more_than_two_item_done: false,
        })
    }

    #[run]
    fn run_empty(&mut self, mut query: Query<'_, (), With<Values>>) {
        assert_iter(query.iter(), [(), (), ()]);
        assert_iter(query.iter_mut(), [(), (), ()]);
        assert_iter(query.iter().rev(), [(), (), ()]);
        assert_iter(query.iter_mut().rev(), [(), (), ()]);
        assert_eq!(query.get(10), None);
        assert_eq!(query.get_mut(10), None);
        assert_eq!(query.get(0), None);
        assert_eq!(query.get_mut(0), None);
        assert_eq!(query.get(1), Some(()));
        assert_eq!(query.get_mut(1), Some(()));
        assert_eq!(query.get_both_mut(1, 2), (Some(()), Some(())));
        self.empty_done = true;
    }

    #[run]
    fn run_one_item(&mut self, mut query: Query<'_, (Entity<'_>,), With<Values>>) {
        assert_iter(query.iter().map(|v| v.0.id()), [2, 1, 3]);
        assert_iter(query.iter_mut().map(|v| v.0.id()), [2, 1, 3]);
        assert_iter(query.iter().rev().map(|v| v.0.id()), [3, 1, 2]);
        assert_iter(query.iter_mut().rev().map(|v| v.0.id()), [3, 1, 2]);
        assert_eq!(query.get(10).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0.id()), None);
        assert_eq!(query.get(0).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(0).map(|v| v.0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0.id()), Some(1));
        assert_eq!(query.get_mut(1).map(|v| v.0.id()), Some(1));
        let (left, right) = query.get_both_mut(1, 2);
        assert_eq!(left.map(|v| v.0.id()), Some(1));
        assert_eq!(right.map(|v| v.0.id()), Some(2));
        self.one_item_done = true;
    }

    #[run]
    fn run_two_items(&mut self, mut query: Query<'_, (Entity<'_>, &Value1), With<Values>>) {
        assert_iter(query.iter().map(|v| v.0.id()), [2, 1]);
        assert_iter(query.iter_mut().map(|v| v.0.id()), [2, 1]);
        assert_iter(query.iter().rev().map(|v| v.0.id()), [1, 2]);
        assert_iter(query.iter_mut().rev().map(|v| v.0.id()), [1, 2]);
        assert_eq!(query.get(10).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0.id()), None);
        assert_eq!(query.get(0).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(0).map(|v| v.0.id()), None);
        assert_eq!(query.get(3).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(3).map(|v| v.0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0.id()), Some(1));
        assert_eq!(query.get_mut(1).map(|v| v.0.id()), Some(1));
        let (left, right) = query.get_both_mut(1, 2);
        assert_eq!(left.map(|v| v.0.id()), Some(1));
        assert_eq!(right.map(|v| v.0.id()), Some(2));
        self.two_item_done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
    }

    #[run]
    fn run_more_than_two_items(
        &mut self,
        mut query: Query<'_, (Entity<'_>, &Value1, &Value2), With<Values>>,
    ) {
        assert_iter(query.iter().map(|v| v.0.id()), [1]);
        assert_iter(query.iter_mut().map(|v| v.0.id()), [1]);
        assert_iter(query.iter().rev().map(|v| v.0.id()), [1]);
        assert_iter(query.iter_mut().rev().map(|v| v.0.id()), [1]);
        assert_eq!(query.get(10).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(10).map(|v| v.0.id()), None);
        assert_eq!(query.get(0).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(0).map(|v| v.0.id()), None);
        assert_eq!(query.get(3).map(|v| v.0.id()), None);
        assert_eq!(query.get_mut(3).map(|v| v.0.id()), None);
        assert_eq!(query.get(1).map(|v| v.0.id()), Some(1));
        assert_eq!(query.get_mut(1).map(|v| v.0.id()), Some(1));
        let (left, right) = query.get_both_mut(1, 2);
        assert_eq!(left.map(|v| v.0.id()), Some(1));
        assert_eq!(right.map(|v| v.0.id()), None);
        self.more_than_two_item_done = true;
    }
}

struct Value1(u32);

struct Value2(u32);

struct Values {
    empty_done: bool,
    one_item_done: bool,
    two_item_done: bool,
    more_than_two_item_done: bool,
}

#[entity]
impl Values {
    fn build(value1: bool, value2: bool) -> impl Built<Self> {
        EntityBuilder::new(Self {
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

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn iteration_on_tuple() {
    let mut app: TestApp = App::new().with_entity(QueryTester::build()).into();
    let values1_id = app.create_entity(Values::build(true, true));
    let values2_id = app.create_entity(Values::build(true, false));
    let values3_id = app.create_entity(Values::build(false, true));
    app.update();
    app.assert_entity(values1_id).has(|v: &Values| {
        assert!(v.empty_done);
        assert!(v.one_item_done);
        assert!(v.two_item_done);
        assert!(v.more_than_two_item_done);
    });
    app.assert_entity(values2_id).has(|v: &Values| {
        assert!(v.empty_done);
        assert!(v.one_item_done);
        assert!(v.two_item_done);
        assert!(!v.more_than_two_item_done);
    });
    app.assert_entity(values3_id).has(|v: &Values| {
        assert!(v.empty_done);
        assert!(!v.one_item_done);
        assert!(!v.two_item_done);
        assert!(!v.more_than_two_item_done);
    });
    app.assert_singleton::<QueryTester>()
        .has(|q: &QueryTester| {
            assert!(q.empty_done);
            assert!(q.one_item_done);
            assert!(q.two_item_done);
            assert!(q.more_than_two_item_done);
        });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_systems_in_parallel() {
    modor_internal::retry!(10, {
        let mut app: TestApp = App::new()
            .with_thread_count(2)
            .with_entity(QueryTester::build())
            .into();
        app.create_entity(Values::build(true, true));
        app.create_entity(Values::build(true, false));
        app.create_entity(Values::build(false, true));
        app.update();
        let start = instant::Instant::now();
        app.update();
        assert!(instant::Instant::now() - start < std::time::Duration::from_millis(150));
    });
}
