use crate::system_params::Value;
use modor::{App, BuiltEntity, EntityBuilder, EntityMut, With, World};

#[modor_test]
fn create_root_entity() {
    App::new()
        .with_entity(tester(|w| {
            w.create_root_entity(IncrementedValue(10));
            w.create_root_entity(Singleton1(20));
            w.create_root_entity(Singleton2(30));
            w.create_root_entity(Singleton1(40));
        }))
        .updated()
        .assert::<With<IncrementedValue>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 10)))
        .assert::<With<Singleton1>>(1, |e| e.has(|s: &Singleton1| assert_eq!(s.0, 40)))
        .assert::<With<Singleton2>>(1, |e| e.has(|s: &Singleton2| assert_eq!(s.0, 30)))
        .updated()
        .assert::<With<IncrementedValue>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 11)));
}

#[modor_test]
fn create_child_entity_with_existing_parent() {
    App::new()
        .with_entity(tester(|w| {
            w.create_child_entity(TESTER_ID, IncrementedValue(10));
            w.create_child_entity(TESTER_ID, Singleton1(20));
            w.create_child_entity(TESTER_ID, Singleton2(30));
            w.create_child_entity(TESTER_ID, Singleton1(40));
        }))
        .updated()
        .assert::<With<IncrementedValue>>(1, |e| {
            e.has(|v: &IncrementedValue| assert_eq!(v.0, 10))
                .has_parent::<With<Tester>>()
        })
        .assert::<With<Singleton1>>(1, |e| {
            e.has(|s: &Singleton1| assert_eq!(s.0, 40))
                .has_parent::<With<Tester>>()
        })
        .assert::<With<Singleton2>>(1, |e| {
            e.has(|s: &Singleton2| assert_eq!(s.0, 30))
                .has_parent::<With<Tester>>()
        })
        .updated()
        .assert::<With<IncrementedValue>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 11)));
}

#[modor_test]
fn create_child_entity_with_missing_parent() {
    App::new()
        .with_entity(tester(|w| {
            w.create_child_entity(MISSING_ID, IncrementedValue(10));
        }))
        .updated()
        .assert::<With<IncrementedValue>>(0, |e| e);
}

#[modor_test]
fn delete_existing_entity() {
    App::new()
        .with_entity(tester_with_child_incremented_value(|w| {
            w.delete_entity(TESTER_ID);
        }))
        .updated()
        .assert::<With<Tester>>(0, |e| e)
        .assert::<With<IncrementedValue>>(0, |e| e);
}

#[modor_test]
fn delete_missing_entity() {
    App::new()
        .with_entity(tester(|w| w.delete_entity(MISSING_ID)))
        .updated()
        .assert::<()>(2, |e| e);
}

#[modor_test]
fn add_component_to_existing_entity_without_component() {
    App::new()
        .with_entity(tester(|w| {
            w.add_component(TESTER_ID, IncrementedValue(10));
            w.add_component(TESTER_ID, Singleton1(20));
            w.add_component(TESTER_ID, Singleton2(30));
            w.add_component(TESTER_ID, Singleton1(40));
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 10)))
        .assert::<With<Tester>>(1, |e| e.has(|v: &Singleton1| assert_eq!(v.0, 40)))
        .assert::<With<Tester>>(1, |e| e.has(|v: &Singleton2| assert_eq!(v.0, 30)))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 11)));
}

#[modor_test]
fn add_component_to_existing_entity_with_component() {
    App::new()
        .with_entity(tester_with_incremented_value(|w| {
            w.add_component(TESTER_ID, IncrementedValue(10));
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 10)))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|v: &IncrementedValue| assert_eq!(v.0, 11)));
}

#[modor_test]
fn add_component_to_missing_entity() {
    App::new()
        .with_entity(tester(|w| {
            w.add_component(MISSING_ID, IncrementedValue(10));
        }))
        .updated()
        .assert::<With<IncrementedValue>>(0, |e| e);
}

#[modor_test]
fn delete_component_of_existing_entity_with_component() {
    // run twice as internal logic is different the second time
    App::new()
        .with_entity(tester_with_incremented_value(|w| {
            w.delete_component::<IncrementedValue>(TESTER_ID);
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has_not::<IncrementedValue>())
        .with_deleted_entities::<()>()
        .with_entity(tester_with_incremented_value(|w| {
            w.delete_component::<IncrementedValue>(TESTER_ID);
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has_not::<IncrementedValue>());
}

#[modor_test]
fn delete_component_of_existing_entity_without_component() {
    App::new()
        .with_entity(tester_with_external_incremented_value(|w| {
            w.delete_component::<IncrementedValue>(TESTER_ID);
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has_not::<IncrementedValue>());
}

#[modor_test]
fn delete_not_registered_component() {
    App::new()
        .with_entity(tester(|w| {
            w.delete_component::<IncrementedValue>(TESTER_ID);
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has_not::<IncrementedValue>());
}

#[modor_test]
fn delete_component_of_missing_entity() {
    App::new()
        .with_entity(tester(|w| {
            w.delete_component::<IncrementedValue>(MISSING_ID);
        }))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has_not::<IncrementedValue>());
}

#[modor_test]
fn retrieve_state() {
    App::new()
        .with_entity(IncrementedValue(10))
        .with_entity(EntityWithAddedComponent(20))
        .with_entity(EntityWithDeletedComponent(30))
        .with_entity(DeletedComponent(40))
        .with_entity(WorldState::default())
        .updated()
        .assert::<With<WorldState>>(1, |e| {
            e.has(|s: &WorldState| assert_eq!(s.transformed_entity_ids, vec![]))
                .has(|s: &WorldState| assert_eq!(s.deleted_entity_ids, vec![]))
        })
        .updated()
        .assert::<With<WorldState>>(1, |e| {
            e.has(|s: &WorldState| assert_eq!(s.transformed_entity_ids, vec![1, 2]))
                .has(|s: &WorldState| assert_eq!(s.deleted_entity_ids, vec![3]))
        })
        .updated()
        .assert::<With<WorldState>>(1, |e| {
            e.has(|s: &WorldState| assert_eq!(s.transformed_entity_ids, vec![]))
                .has(|s: &WorldState| assert_eq!(s.deleted_entity_ids, vec![]))
        });
}

#[modor_test(disabled(wasm))]
fn run_systems_in_parallel() {
    assert!(!are_systems_run_in_parallel!((), World<'_>));
}

const TESTER_ID: usize = 1;
const MISSING_ID: usize = 100;

fn tester(test_fn: fn(&mut World<'_>)) -> impl BuiltEntity {
    EntityBuilder::new().child_component(Tester::new(test_fn))
}

fn tester_with_external_incremented_value(test_fn: fn(&mut World<'_>)) -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Tester::new(test_fn))
        .child_component(IncrementedValue(0))
}

fn tester_with_child_incremented_value(test_fn: fn(&mut World<'_>)) -> impl BuiltEntity {
    EntityBuilder::new().child_entity(
        EntityBuilder::new()
            .component(Tester::new(test_fn))
            .child_component(IncrementedValue(0)),
    )
}

fn tester_with_incremented_value(test_fn: fn(&mut World<'_>)) -> impl BuiltEntity {
    EntityBuilder::new().child_entity(
        EntityBuilder::new()
            .component(Tester::new(test_fn))
            .component(IncrementedValue(0)),
    )
}

#[derive(Component)]
struct Tester {
    test_fn: fn(&mut World<'_>),
    is_done: bool,
}

#[systems]
impl Tester {
    fn new(test_fn: fn(&mut World<'_>)) -> Self {
        Self {
            test_fn,
            is_done: false,
        }
    }

    #[run]
    #[allow(clippy::redundant_closure_call)]
    fn update(&mut self, mut world: World<'_>) {
        if !self.is_done {
            (self.test_fn)(&mut world);
            self.is_done = true;
        }
    }
}

#[derive(SingletonComponent, Default)]
struct WorldState {
    transformed_entity_ids: Vec<usize>,
    deleted_entity_ids: Vec<usize>,
}

#[systems]
impl WorldState {
    #[run]
    fn update(&mut self, world: World<'_>) {
        self.transformed_entity_ids = world.transformed_entity_ids().collect();
        self.deleted_entity_ids = world.deleted_entity_ids().collect();
    }
}

#[derive(Component)]
struct IncrementedValue(u32);

#[systems]
impl IncrementedValue {
    #[run]
    fn update(&mut self) {
        self.0 += 1;
    }
}

#[derive(Component)]
struct EntityWithAddedComponent(u32);

#[systems]
impl EntityWithAddedComponent {
    #[run]
    fn update(mut entity: EntityMut<'_>) {
        entity.add_component(Value(10));
    }
}

#[derive(Component)]
struct EntityWithDeletedComponent(u32);

#[systems]
impl EntityWithDeletedComponent {
    #[run]
    fn update(mut entity: EntityMut<'_>) {
        entity.delete_component::<Self>();
    }
}

#[derive(Component)]
struct DeletedComponent(u32);

#[systems]
impl DeletedComponent {
    #[run]
    fn update(mut entity: EntityMut<'_>) {
        entity.delete();
    }
}

#[derive(SingletonComponent, NoSystem)]
struct Singleton1(u32);

#[derive(SingletonComponent, NoSystem)]
struct Singleton2(u32);
