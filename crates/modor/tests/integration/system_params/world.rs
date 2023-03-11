use crate::system_params::Text;
use fxhash::FxHashSet;
use modor::{App, BuiltEntity, Entity, EntityBuilder, LevelFilter, Query, With, World};

#[derive(Component, NoSystem)]
struct Id(u32);

#[derive(Component)]
struct EntityToDelete;

impl EntityToDelete {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }
}

#[systems]
impl EntityToDelete {
    #[run]
    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
        world.delete_entity(100); // not existing entity
    }
}

#[derive(Component, NoSystem)]
struct DeletedChild;

#[derive(Component)]
struct ParentEntityToDelete;

#[systems]
impl ParentEntityToDelete {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Id(id))
            .with_child(DeletedChild)
    }

    #[run]
    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[derive(Component, NoSystem)]
struct ParentOfEntityToDelete;

impl ParentOfEntityToDelete {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Id(id))
            .with_child(EntityBuilder::new().with(Id(id)).with(EntityToDelete))
    }
}

#[derive(Component)]
struct EntityWithMissingComponentAdded;

#[systems]
impl EntityWithMissingComponentAdded {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }

    #[run]
    fn add_component(parent: &Id, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), Text(format!("id: {}", parent.0)));
        world.add_component(101, Text(format!("id: {}", parent.0))); // not existing entity
    }
}

#[derive(Component)]
struct EntityWithExistingComponentAdded;

#[systems]
impl EntityWithExistingComponentAdded {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Text(String::from("empty")))
            .with(Id(id))
    }

    #[run]
    fn add_component(parent: &Id, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), Text(format!("id: {}", parent.0)));
    }
}

#[derive(SingletonComponent)]
struct SingletonWithComponentAdded;

#[systems]
impl SingletonWithComponentAdded {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }

    #[run]
    fn add_component(parent: &Id, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), Text(format!("id: {}", parent.0)));
    }
}

#[derive(Component)]
struct EntityWithExistingComponentDeleted;

#[systems]
impl EntityWithExistingComponentDeleted {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Id(id))
            .with(Text(String::from("existing")))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<Text>(entity.id());
    }
}

#[derive(Component)]
struct EntityWithMissingComponentDeleted;

#[systems]
impl EntityWithMissingComponentDeleted {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<Text>(entity.id());
    }
}

#[derive(Component, NoSystem)]
struct NotRegisteredComponent;

#[derive(Component)]
struct EntityWithNotRegisteredComponentTypeDeleted;

#[systems]
impl EntityWithNotRegisteredComponentTypeDeleted {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<NotRegisteredComponent>(entity.id());
    }
}

#[derive(Component)]
struct EntityWithAddedChild;

#[systems]
impl EntityWithAddedChild {
    fn build(id: u32) -> impl BuiltEntity {
        EntityBuilder::new().with(Self).with(Id(id))
    }

    #[run]
    fn create_root_entity(mut world: World<'_>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
        world.create_root_entity(NewRootEntity(80));
    }

    #[run]
    fn create_child_entity_from_existing_parent(entity: Entity<'_>, mut world: World<'_>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
        world.create_child_entity(entity.id(), NewChildEntity(70));
    }

    #[run]
    fn create_child_entity_from_missing_parent(mut world: World<'_>) {
        world.create_child_entity(99999, NewChildEntity(70));
    }
}

#[derive(Component, NoSystem)]
struct NewRootEntity(u32);

#[derive(Component, NoSystem)]
struct NewChildEntity(u32);

#[derive(SingletonComponent, Default)]
struct WorldState {
    parents: Vec<Option<u32>>,
    deleted_parent_ids: FxHashSet<u32>,
    transformed_entity_ids: FxHashSet<u32>,
}

#[systems]
impl WorldState {
    #[run]
    fn update(&mut self, world: World<'_>, query: Query<'_, (Entity<'_>, &Id)>) {
        for (entity, parent) in query.iter() {
            for _ in self.parents.len()..=entity.id() {
                self.parents.push(None);
            }
            self.parents[entity.id()] = Some(parent.0);
        }
        self.deleted_parent_ids = world
            .deleted_entity_ids()
            .filter_map(|i| self.parents.get(i).copied().flatten())
            .collect();
        self.transformed_entity_ids = world
            .transformed_entity_ids()
            .filter_map(|i| self.parents.get(i).copied().flatten())
            .collect();
    }
}

#[test]
fn use_world() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(WorldState::default())
        .with_entity(EntityToDelete::build(10))
        .with_entity(ParentEntityToDelete::build(11))
        .with_entity(ParentOfEntityToDelete::build(12))
        .with_entity(EntityWithMissingComponentAdded::build(20))
        .with_entity(EntityWithExistingComponentAdded::build(21))
        .with_entity(SingletonWithComponentAdded::build(22))
        .with_entity(EntityWithExistingComponentDeleted::build(30))
        .with_entity(EntityWithExistingComponentDeleted::build(31))
        .with_entity(EntityWithMissingComponentDeleted::build(40))
        .with_entity(EntityWithNotRegisteredComponentTypeDeleted::build(50))
        .with_entity(EntityWithAddedChild::build(60))
        .updated()
        .assert::<With<WorldState>>(1, |e| {
            e.has(|s: &WorldState| {
                assert!(s.deleted_parent_ids.is_empty());
                assert!(s.transformed_entity_ids.is_empty());
            })
        })
        .assert::<With<EntityToDelete>>(0, |e| e)
        .assert::<With<ParentEntityToDelete>>(0, |e| e)
        .assert::<With<DeletedChild>>(0, |e| e)
        .assert::<With<ParentOfEntityToDelete>>(1, |e| e.child_count(0))
        .assert::<With<EntityWithMissingComponentAdded>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 20))
                .has(|c: &Text| assert_eq!(c.0, "id: 20"))
        })
        .assert::<With<EntityWithExistingComponentAdded>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 21))
                .has(|c: &Text| assert_eq!(c.0, "id: 21"))
        })
        .assert::<With<SingletonWithComponentAdded>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 22))
                .has(|c: &Text| assert_eq!(c.0, "id: 22"))
        })
        .assert::<With<EntityWithExistingComponentDeleted>>(2, |e| {
            e.has_not::<Text>()
                .any()
                .has(|c: &Id| assert_eq!(c.0, 30))
                .has(|c: &Id| assert_eq!(c.0, 31))
        })
        .assert::<With<EntityWithMissingComponentDeleted>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 40)).has_not::<Text>()
        })
        .assert::<With<EntityWithNotRegisteredComponentTypeDeleted>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 50)).has_not::<Text>()
        })
        .assert::<With<EntityWithAddedChild>>(1, |e| {
            e.has(|c: &Id| assert_eq!(c.0, 60))
                .has_not::<Text>()
                .child_count(1)
        })
        .assert::<With<NewChildEntity>>(1, |e| {
            e.has(|e: &NewChildEntity| assert_eq!(e.0, 70))
                .has_parent::<With<EntityWithAddedChild>>()
        })
        .assert::<With<NewRootEntity>>(1, |e| e.has(|e: &NewRootEntity| assert_eq!(e.0, 80)))
        .updated()
        .assert::<With<WorldState>>(1, |e| {
            e.has(|s: &WorldState| {
                assert_eq!(
                    s.deleted_parent_ids,
                    [10, 11, 12].into_iter().collect::<FxHashSet<_>>()
                );
                assert_eq!(
                    s.transformed_entity_ids,
                    [20, 22, 30, 31].into_iter().collect::<FxHashSet<_>>()
                );
            })
        });
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_systems_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(EntityWithAddedChild::build(60))
        .updated();
    assert!(start.elapsed() > std::time::Duration::from_millis(200));
}
