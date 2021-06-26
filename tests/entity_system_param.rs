use modor::*;

struct Parent(u32);

impl EntityMainComponent for Parent {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.with_self(Self(id))
    }
}

struct EntityToDelete;

impl EntityMainComponent for EntityToDelete {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::delete));
    }
}

impl EntityToDelete {
    fn delete(mut entity: Entity<'_>) {
        entity.delete();
    }
}

struct EntityWithAddedComponent;

impl EntityMainComponent for EntityWithAddedComponent {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::add_component));
    }
}

impl EntityWithAddedComponent {
    fn add_component(parent: &Parent, mut entity: Entity<'_>) {
        entity.add_component(format!("id: {}", parent.0));
    }
}

struct EntityWithExistingComponentDeleted;

impl EntityMainComponent for EntityWithExistingComponentDeleted {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder
            .inherit_from::<Parent>(id)
            .with(String::from("existing"))
            .with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::delete_component));
    }
}

impl EntityWithExistingComponentDeleted {
    fn delete_component(mut entity: Entity<'_>) {
        entity.delete_component::<String>();
    }
}

struct EntityWithMissingComponentDeleted;

impl EntityMainComponent for EntityWithMissingComponentDeleted {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::delete_component));
    }
}

impl EntityWithMissingComponentDeleted {
    fn delete_component(mut entity: Entity<'_>) {
        entity.delete_component::<String>();
    }
}

struct EntityWithNotRegisteredComponentTypeDeleted;

impl EntityMainComponent for EntityWithNotRegisteredComponentTypeDeleted {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::delete_component));
    }
}

impl EntityWithNotRegisteredComponentTypeDeleted {
    fn delete_component(mut entity: Entity<'_>) {
        entity.delete_component::<i64>();
    }
}

fn build_group(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<EntityToDelete>(10);
    builder.with_entity::<EntityWithAddedComponent>(20);
    builder.with_entity::<EntityWithExistingComponentDeleted>(30);
    builder.with_entity::<EntityWithMissingComponentDeleted>(40);
    builder.with_entity::<EntityWithNotRegisteredComponentTypeDeleted>(50);
}

#[test]
fn init() {
    let mut application = Application::new().with_group(build_group);

    let mut components = Vec::new();
    application.run(system_once!(
        |p: &Parent, s: Option<&String>| components.push((p.0, s.cloned()))
    ));
    components.sort_unstable();
    let expected_components = [
        (10, None),
        (20, None),
        (30, Some("existing".into())),
        (40, None),
        (50, None),
    ];
    assert_eq!(components, expected_components);
}

#[test]
fn update() {
    let mut application = Application::new().with_group(build_group);

    application.update();

    let mut components = Vec::new();
    application.run(system_once!(
        |p: &Parent, s: Option<&String>| components.push((p.0, s.cloned()))
    ));
    components.sort_unstable();
    let expected_components = [
        (20, Some("id: 20".into())),
        (30, None),
        (40, None),
        (50, None),
    ];
    assert_eq!(components, expected_components);
}
