use crate::system_params::assert_iter;
use modor::{App, Built, Entity, EntityBuilder, Filter, Query, SingleMut, With};

struct QueryTester {
    done: bool,
}

#[singleton]
impl QueryTester {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self { done: false })
    }

    #[run]
    fn collect(&mut self, mut query: Query<'_, (Entity<'_>, Filter<With<Number>>)>) {
        assert_iter(query.iter().map(|e| e.0.id()), [5, 2, 4, 6]);
        assert_iter(query.iter_mut().map(|e| e.0.id()), [5, 2, 4, 6]);
        assert_iter(query.iter().rev().map(|e| e.0.id()), [6, 4, 2, 5]);
        assert_iter(query.iter_mut().rev().map(|e| e.0.id()), [6, 4, 2, 5]);
        assert_eq!(query.get(10).map(|e| e.0.id()), None);
        assert_eq!(query.get_mut(10).map(|e| e.0.id()), None);
        assert_eq!(query.get(3).map(|e| e.0.id()), None);
        assert_eq!(query.get_mut(3).map(|e| e.0.id()), None);
        assert_eq!(query.get(4).map(|e| e.0.id()), Some(4));
        assert_eq!(query.get_mut(4).map(|e| e.0.id()), Some(4));
        let (left, right) = query.get_both_mut(4, 2);
        assert_eq!(left.map(|e| e.0.id()), Some(4));
        assert_eq!(right.map(|e| e.0.id()), Some(2));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(200));
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
            e.has(|c: &StreamCollector| assert_eq!(c.0, [5, 2, 4, 6]))
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
        assert!(start.elapsed() < std::time::Duration::from_millis(250));
    });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_entity() {
    App::new()
        .with_entity(Parent::build())
        .updated()
        .assert::<With<Parent>>(1, |e| e.has(|p: &Parent| assert!(p.done)));
}
