use modor::testing::TestApp;
use modor::{system, Built, Entity, EntityBuilder, EntityMainComponent, SystemRunner, World};

#[derive(PartialEq, Debug)]
struct Parent(u32);

impl Parent {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

impl EntityMainComponent for Parent {
    type Type = ();
}

struct EntityToDelete;

impl EntityToDelete {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

impl EntityMainComponent for EntityToDelete {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete))
    }
}

struct EntityWithAddedComponent;

impl EntityWithAddedComponent {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
    }
}

impl EntityMainComponent for EntityWithAddedComponent {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::add_component))
    }
}

struct EntityWithExistingComponentDeleted;

impl EntityWithExistingComponentDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .inherit_from(Parent::build(id))
            .with(String::from("existing"))
    }

    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

impl EntityMainComponent for EntityWithExistingComponentDeleted {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

struct EntityWithMissingComponentDeleted;

impl EntityWithMissingComponentDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

impl EntityMainComponent for EntityWithMissingComponentDeleted {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

struct EntityWithNotRegisteredComponentTypeDeleted;

impl EntityWithNotRegisteredComponentTypeDeleted {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Parent::build(id))
    }

    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<i64>(entity.id());
        world.create_root_entity(NewRootEntity::build(10));
        world.create_child_entity(entity.id(), NewChildEntity::build(20));
    }
}

impl EntityMainComponent for EntityWithNotRegisteredComponentTypeDeleted {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

struct NewRootEntity(u32);

impl NewRootEntity {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

impl EntityMainComponent for NewRootEntity {
    type Type = ();
}

struct NewChildEntity(u32);

impl NewChildEntity {
    fn build(id: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(id))
    }
}

impl EntityMainComponent for NewChildEntity {
    type Type = ();
}

#[test]
fn use_world() {
    let mut app = TestApp::new();
    let entity1_id = app.create_entity(EntityToDelete::build(10));
    let entity2_id = app.create_entity(EntityWithAddedComponent::build(20));
    let entity3_id = app.create_entity(EntityWithExistingComponentDeleted::build(30));
    let entity4_id = app.create_entity(EntityWithMissingComponentDeleted::build(40));
    let entity5_id = app.create_entity(EntityWithNotRegisteredComponentTypeDeleted::build(50));
    app.assert_entity(entity1_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(10)))
        .has_not::<String>();
    app.assert_entity(entity2_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(20)))
        .has_not::<String>();
    app.assert_entity(entity3_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(30)))
        .has::<String, _>(|c| assert_eq!(c, "existing"));
    app.assert_entity(entity4_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(40)))
        .has_not::<String>();
    app.assert_entity(entity5_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(50)))
        .has_not::<String>()
        .has_children(|c| assert_eq!(c.len(), 0));
    app.assert_entity(entity5_id + 1).does_not_exist();

    app.update();
    app.assert_entity(entity1_id).does_not_exist();
    app.assert_entity(entity2_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(20)))
        .has::<String, _>(|c| assert_eq!(c, "id: 20"));
    app.assert_entity(entity3_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(30)))
        .has_not::<String>();
    app.assert_entity(entity4_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(40)))
        .has_not::<String>();
    app.assert_entity(entity5_id)
        .has::<Parent, _>(|c| assert_eq!(c, &Parent(50)))
        .has_not::<String>()
        .has_children(|c| {
            assert_eq!(c.len(), 1);
            app.assert_entity(c[0])
                .has::<NewChildEntity, _>(|e| assert_eq!(e.0, 20));
        });
    app.assert_entity(entity5_id + 1)
        .has::<NewChildEntity, _>(|e| assert_eq!(e.0, 20));
    app.assert_entity(entity5_id + 2)
        .has::<NewRootEntity, _>(|e| assert_eq!(e.0, 10));
}
