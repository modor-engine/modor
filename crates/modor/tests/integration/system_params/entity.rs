use crate::system_params::{assert_iter, Number, OtherNumber};
use modor::{App, BuiltEntity, Entity, EntityBuilder, Filter, Query, SingleMut, With};

#[derive(SingletonComponent, Default)]
struct QueryTester {
    done: bool,
}

#[systems]
impl QueryTester {
    #[run]
    fn collect(&mut self, mut query: Query<'_, (Entity<'_>, Filter<With<Number>>)>) {
        assert_iter(query.iter().map(|e| e.0.id()), [3, 5, 6, 7]);
        assert_iter(query.iter_mut().map(|e| e.0.id()), [3, 5, 6, 7]);
        assert_iter(query.iter().rev().map(|e| e.0.id()), [7, 6, 5, 3]);
        assert_iter(query.iter_mut().rev().map(|e| e.0.id()), [7, 6, 5, 3]);
        assert_eq!(query.get(10).map(|e| e.0.id()), None);
        assert_eq!(query.get_mut(10).map(|e| e.0.id()), None);
        assert_eq!(query.get(4).map(|e| e.0.id()), None);
        assert_eq!(query.get_mut(4).map(|e| e.0.id()), None);
        assert_eq!(query.get(5).map(|e| e.0.id()), Some(5));
        assert_eq!(query.get_mut(5).map(|e| e.0.id()), Some(5));
        let (left, right) = query.get_both_mut(5, 3);
        assert_eq!(left.map(|e| e.0.id()), Some(5));
        assert_eq!(right.map(|e| e.0.id()), Some(3));
        self.done = true;
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(200));
    }
}

#[derive(SingletonComponent, NoSystem, Default)]
struct EntityIds(Vec<usize>);

#[derive(Component)]
struct RegisteredNumber;

#[systems]
impl RegisteredNumber {
    #[run]
    fn collect(entity: Entity<'_>, mut ids: SingleMut<'_, EntityIds>) {
        ids.0.push(entity.id());
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(50));
    }
}

#[derive(SingletonComponent)]
struct Parent {
    done: bool,
}

#[systems]
impl Parent {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self { done: false })
            .with_child(Number::build(20))
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

fn entities() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(QueryTester::default())
        .with_child(EntityIds::default())
        .with_child(Number::build(1).with(RegisteredNumber))
        .with_child(OtherNumber::build(10))
        .with_child(Number::build(2).with(RegisteredNumber))
        .with_child(Number::build_without_value().with(RegisteredNumber))
        .with_child(Number::build_with_additional_component(3).with(RegisteredNumber))
}

#[modor_test]
fn iterate_on_component_reference() {
    App::new()
        .with_entity(entities())
        .updated()
        .assert::<With<EntityIds>>(1, |e| e.has(|c: &EntityIds| assert_eq!(c.0, [3, 5, 6, 7])))
        .assert::<With<QueryTester>>(1, |e| e.has(|c: &QueryTester| assert!(c.done)));
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

#[modor_test]
fn use_entity() {
    App::new()
        .with_entity(Parent::build())
        .updated()
        .assert::<With<Parent>>(1, |e| e.has(|p: &Parent| assert!(p.done)));
}
