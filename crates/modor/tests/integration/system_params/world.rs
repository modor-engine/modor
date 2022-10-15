use modor::{App, Built, Entity, EntityBuilder, With, World};

struct Parent(u32);

#[entity]
impl Parent {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

struct EntityToDelete;

#[entity]
impl EntityToDelete {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
        world.delete_entity(100); // not existing entity
    }
}

struct DeletedChild;

#[entity]
impl DeletedChild {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }
}

struct ParentEntityToDelete;

#[entity]
impl ParentEntityToDelete {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Parent::build(id))
            .with_child(DeletedChild::build())
    }

    #[run]
    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

struct ParentOfEntityToDelete;

#[entity]
impl ParentOfEntityToDelete {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Parent::build(id))
            .with_child(EntityToDelete::build(id))
    }
}

struct EntityWithMissingComponentAdded;

#[entity]
impl EntityWithMissingComponentAdded {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
        world.add_component(101, format!("id: {}", parent.0)); // not existing entity
    }
}

struct EntityWithExistingComponentAdded;

#[entity]
impl EntityWithExistingComponentAdded {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(String::from("empty"))
            .inherit_from(Parent::build(id))
    }

    #[run]
    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
    }
}

struct SingletonWithComponentAdded;

#[singleton]
impl SingletonWithComponentAdded {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
    }
}

struct UnregisteredSingletonWithComponentAdded;

#[singleton]
impl UnregisteredSingletonWithComponentAdded {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(SingletonWithComponentAdded)
            .inherit_from(Parent::build(id))
    }

    #[run]
    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
    }
}

struct EntityWithExistingComponentDeleted;

#[entity]
impl EntityWithExistingComponentDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Parent::build(id))
            .with(String::from("existing"))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

struct EntityWithMissingComponentDeleted;

#[entity]
impl EntityWithMissingComponentDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

struct EntityWithNotRegisteredComponentTypeDeleted;

#[entity]
impl EntityWithNotRegisteredComponentTypeDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<i64>(entity.id());
    }
}

struct EntityWithAddedChild;

#[entity]
impl EntityWithAddedChild {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    #[run]
    fn create_root_entity(mut world: World<'_>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
        world.create_root_entity(NewRootEntity::build(80));
    }

    #[run]
    fn create_child_entity(entity: Entity<'_>, mut world: World<'_>) {
        #[cfg(not(target_arch = "wasm32"))]
        spin_sleep::sleep(std::time::Duration::from_millis(100));
        world.create_child_entity(entity.id(), NewChildEntity::build(70));
    }
}

struct NewRootEntity(u32);

#[entity]
impl NewRootEntity {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

struct NewChildEntity(u32);

#[entity]
impl NewChildEntity {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_world() {
    App::new()
        .with_entity(EntityToDelete::build(10))
        .with_entity(ParentEntityToDelete::build(11))
        .with_entity(ParentOfEntityToDelete::build(12))
        .with_entity(EntityWithMissingComponentAdded::build(20))
        .with_entity(EntityWithExistingComponentAdded::build(21))
        .with_entity(SingletonWithComponentAdded::build(22))
        .with_entity(UnregisteredSingletonWithComponentAdded::build(23))
        .with_entity(EntityWithExistingComponentDeleted::build(30))
        .with_entity(EntityWithExistingComponentDeleted::build(31))
        .with_entity(EntityWithMissingComponentDeleted::build(40))
        .with_entity(EntityWithNotRegisteredComponentTypeDeleted::build(50))
        .with_entity(EntityWithAddedChild::build(60))
        .updated()
        .assert::<With<EntityToDelete>>(0, |e| e)
        .assert::<With<ParentEntityToDelete>>(0, |e| e)
        .assert::<With<DeletedChild>>(0, |e| e)
        .assert::<With<ParentOfEntityToDelete>>(1, |e| e.child_count(0))
        .assert::<With<EntityWithMissingComponentAdded>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 20))
                .has(|c: &String| assert_eq!(c, "id: 20"))
        })
        .assert::<With<EntityWithExistingComponentAdded>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 21))
                .has(|c: &String| assert_eq!(c, "id: 21"))
        })
        .assert::<With<SingletonWithComponentAdded>>(2, |e| {
            e.any()
                .has(|c: &Parent| assert_eq!(c.0, 22))
                .has(|c: &Parent| assert_eq!(c.0, 23))
                .has(|c: &String| assert_eq!(c, "id: 22"))
                .has(|c: &String| assert_eq!(c, "id: 23"))
        })
        .assert::<With<UnregisteredSingletonWithComponentAdded>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 23))
                .has(|c: &String| assert_eq!(c, "id: 23"))
        })
        .assert::<With<EntityWithExistingComponentDeleted>>(2, |e| {
            e.has_not::<String>()
                .any()
                .has(|c: &Parent| assert_eq!(c.0, 30))
                .has(|c: &Parent| assert_eq!(c.0, 31))
        })
        .assert::<With<EntityWithMissingComponentDeleted>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 40)).has_not::<String>()
        })
        .assert::<With<EntityWithNotRegisteredComponentTypeDeleted>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 50)).has_not::<String>()
        })
        .assert::<With<EntityWithAddedChild>>(1, |e| {
            e.has(|c: &Parent| assert_eq!(c.0, 60))
                .has_not::<String>()
                .child_count(1)
        })
        .assert::<With<NewChildEntity>>(1, |e| {
            e.has(|e: &NewChildEntity| assert_eq!(e.0, 70))
                .has_parent::<With<EntityWithAddedChild>>()
        })
        .assert::<With<NewRootEntity>>(1, |e| e.has(|e: &NewRootEntity| assert_eq!(e.0, 80)));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_must_use)]
fn run_systems_in_parallel() {
    let start = instant::Instant::now();
    App::new()
        .with_thread_count(2)
        .with_entity(EntityWithAddedChild::build(60))
        .updated();
    assert!(instant::Instant::now() - start > std::time::Duration::from_millis(200));
}
