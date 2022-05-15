use modor::testing::TestApp;
use modor::{Built, Entity, EntityBuilder, World};

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
    }
}

struct EntityWithAddedComponent;

#[entity]
impl EntityWithAddedComponent {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
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
    let mut app = TestApp::new();
    let entity1_id = app.create_entity(EntityToDelete::build(10));
    let entity2_id = app.create_entity(EntityToDelete::build(11));
    let entity3_id = app.create_entity(EntityWithAddedComponent::build(20));
    let entity4_id = app.create_entity(EntityWithExistingComponentDeleted::build(30));
    let entity5_id = app.create_entity(EntityWithMissingComponentDeleted::build(40));
    let entity6_id = app.create_entity(EntityWithNotRegisteredComponentTypeDeleted::build(50));
    let entity7_id = app.create_entity(EntityWithAddedChild::build(60));
    app.update();
    app.assert_entity(entity1_id).does_not_exist();
    app.assert_entity(entity2_id).does_not_exist();
    app.assert_entity(entity3_id)
        .has(|c: &Parent| assert_eq!(c.0, 20))
        .has(|c: &String| assert_eq!(c, "id: 20"));
    app.assert_entity(entity4_id)
        .has(|c: &Parent| assert_eq!(c.0, 30))
        .has_not::<String>();
    app.assert_entity(entity5_id)
        .has(|c: &Parent| assert_eq!(c.0, 40))
        .has_not::<String>();
    app.assert_entity(entity6_id)
        .has(|c: &Parent| assert_eq!(c.0, 50))
        .has_not::<String>();
    app.assert_entity(entity7_id)
        .has(|c: &Parent| assert_eq!(c.0, 60))
        .has_not::<String>()
        .has_children(|c| {
            assert_eq!(c, vec![entity7_id + 1]);
            app.assert_entity(c[0])
                .has(|e: &NewChildEntity| assert_eq!(e.0, 70));
        });
    app.assert_entity(entity7_id + 2)
        .has(|e: &NewRootEntity| assert_eq!(e.0, 80));
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn run_systems_in_parallel() {
    let mut app: TestApp = modor::App::new()
        .with_thread_count(2)
        .with_entity(EntityWithAddedChild::build(60))
        .into();
    let start = instant::Instant::now();
    app.update();
    assert!(instant::Instant::now() - start > std::time::Duration::from_millis(200));
}
