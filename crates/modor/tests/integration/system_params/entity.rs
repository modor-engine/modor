use crate::system_params::assert_iter;
use instant::Instant;
use modor::testing::TestApp;
use modor::{App, Built, Entity, EntityBuilder, Query, SingleMut, With};
use std::time::Duration;

struct QueryTester {
    done: bool,
}

#[singleton]
impl QueryTester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done: false })
    }

    #[run]
    fn collect(&mut self, mut query: Query<'_, Entity<'_>, With<Number>>) {
        assert_iter(query.iter().map(Entity::id), [5, 2, 4, 6]);
        assert_iter(query.iter_mut().map(Entity::id), [5, 2, 4, 6]);
        assert_iter(query.iter().rev().map(Entity::id), [6, 4, 2, 5]);
        assert_iter(query.iter_mut().rev().map(Entity::id), [6, 4, 2, 5]);
        assert_eq!(query.get(10).map(Entity::id), None);
        assert_eq!(query.get_mut(10).map(Entity::id), None);
        assert_eq!(query.get(3).map(Entity::id), None);
        assert_eq!(query.get_mut(3).map(Entity::id), None);
        assert_eq!(query.get(4).map(Entity::id), Some(4));
        assert_eq!(query.get_mut(4).map(Entity::id), Some(4));
        let (left, right) = query.get_both_mut(4, 2);
        assert_eq!(left.map(Entity::id), Some(4));
        assert_eq!(right.map(Entity::id), Some(2));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(Duration::from_millis(200));
    }
}

struct StreamCollector(Vec<usize>);

#[singleton]
impl StreamCollector {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(vec![]))
    }
}

struct Value(u32);

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
            .with(String::from("other"))
    }

    #[run]
    fn collect(entity: Entity<'_>, mut collector: SingleMut<'_, StreamCollector>) {
        collector.0.push(entity.id());
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(Duration::from_millis(50));
    }
}

struct OtherNumber;

#[entity]
impl OtherNumber {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Value(value))
    }
}

struct Parent {
    done: bool,
}

#[singleton]
impl Parent {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done: false }).with_child(Number::build(20))
    }

    #[run]
    fn update(&mut self, entity: Entity<'_>) {
        assert_eq!(entity.children().len(), 1);
        let child = entity.children().next().unwrap();
        assert_eq!(entity.id(), 0);
        assert_eq!(child.id(), 1);
        assert_eq!(entity.depth(), 0);
        assert_eq!(child.depth(), 1);
        assert_eq!(entity.parent().map(Entity::id), None);
        assert_eq!(child.parent().map(Entity::id), Some(0));
        assert_eq!(child.children().len(), 0);
        self.done = true;
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn iterate_on_component_reference() {
    let mut app: TestApp = App::new()
        .with_entity(QueryTester::build())
        .with_entity(StreamCollector::build())
        .with_entity(Number::build(1))
        .with_entity(OtherNumber::build(10))
        .with_entity(Number::build(2))
        .with_entity(Number::build_without_value())
        .with_entity(Number::build_with_additional_component(3))
        .into();
    app.update();
    app.assert_singleton::<StreamCollector>()
        .has(|c: &StreamCollector| assert_eq!(c.0, [5, 2, 4, 6]));
    app.assert_singleton::<QueryTester>()
        .has(|c: &QueryTester| assert!(c.done));
}

#[test]
fn run_systems_in_parallel() {
    modor_internal::retry!(10, {
        let mut app: TestApp = App::new()
            .with_thread_count(2)
            .with_entity(QueryTester::build())
            .with_entity(StreamCollector::build())
            .with_entity(Number::build(1))
            .with_entity(OtherNumber::build(10))
            .with_entity(Number::build(2))
            .with_entity(Number::build_without_value())
            .with_entity(Number::build_with_additional_component(3))
            .into();
        let start = Instant::now();
        app.update();
        assert!(Instant::now() - start < Duration::from_millis(250));
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_entity() {
    let mut app: TestApp = App::new().with_entity(Parent::build()).into();
    app.update();
    app.assert_singleton::<Parent>()
        .has(|p: &Parent| assert!(p.done));
}
