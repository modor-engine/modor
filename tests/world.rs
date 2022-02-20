use modor::testing::TestApp;
use modor::{system, Built, Entity, EntityBuilder, EntityMainComponent, SystemRunner, World};

#[derive(PartialEq, Debug)]
struct Parent(u32);

impl EntityMainComponent for Parent {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.with_self(Self(id))
    }
}

struct EntityToDelete;

impl EntityMainComponent for EntityToDelete {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete))
    }
}

impl EntityToDelete {
    fn delete(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

struct EntityWithAddedComponent;

impl EntityMainComponent for EntityWithAddedComponent {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::add_component))
    }
}

impl EntityWithAddedComponent {
    fn add_component(parent: &Parent, entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), format!("id: {}", parent.0));
    }
}

struct EntityWithExistingComponentDeleted;

impl EntityMainComponent for EntityWithExistingComponentDeleted {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder
            .inherit_from::<Parent>(id)
            .with(String::from("existing"))
            .with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

impl EntityWithExistingComponentDeleted {
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

struct EntityWithMissingComponentDeleted;

impl EntityMainComponent for EntityWithMissingComponentDeleted {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

impl EntityWithMissingComponentDeleted {
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<String>(entity.id());
    }
}

struct EntityWithNotRegisteredComponentTypeDeleted;

impl EntityMainComponent for EntityWithNotRegisteredComponentTypeDeleted {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::delete_component))
    }
}

impl EntityWithNotRegisteredComponentTypeDeleted {
    fn delete_component(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_component::<i64>(entity.id());
    }
}

#[test]
fn use_world() {
    let mut app = TestApp::new();
    let entity1_id = app.create_entity::<EntityToDelete>(10);
    let entity2_id = app.create_entity::<EntityWithAddedComponent>(20);
    let entity3_id = app.create_entity::<EntityWithExistingComponentDeleted>(30);
    let entity4_id = app.create_entity::<EntityWithMissingComponentDeleted>(40);
    let entity5_id = app.create_entity::<EntityWithNotRegisteredComponentTypeDeleted>(50);
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
        .has_not::<String>();

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
        .has_not::<String>();
}
