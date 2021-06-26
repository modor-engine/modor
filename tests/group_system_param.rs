use modor::*;

struct Parent(u32);

impl EntityMainComponent for Parent {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.with_self(Self(id))
    }
}

struct GroupReplacer;

impl EntityMainComponent for GroupReplacer {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::replace_group));
    }
}

impl GroupReplacer {
    fn replace_group(mut group: Group<'_>) {
        group.replace(|b| {
            b.with_entity::<Self>(30);
            b.with_entity::<Self>(40);
        });
    }
}

struct GroupDeleter;

impl EntityMainComponent for GroupDeleter {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::delete_group));
    }
}

impl GroupDeleter {
    fn delete_group(mut group: Group<'_>) {
        group.delete();
    }
}

struct EntityAdder;

impl EntityMainComponent for EntityAdder {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.inherit_from::<Parent>(id).with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::add_entity));
    }
}

impl EntityAdder {
    fn add_entity(mut group: Group<'_>) {
        group.create_entity::<Self>(220);
    }
}

fn build_group_to_replace(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<GroupReplacer>(10);
    builder.with_entity::<GroupReplacer>(20);
}

fn build_group_to_delete(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<GroupDeleter>(110);
    builder.with_entity::<GroupDeleter>(120);
}

fn build_group_with_added_entity(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<EntityAdder>(210);
}

#[test]
fn init() {
    let mut application = Application::new()
        .with_group(build_group_to_replace)
        .with_group(build_group_to_delete)
        .with_group(build_group_with_added_entity);

    let mut ids = Vec::new();
    application.run(system_once!(|p: &Parent| ids.push(p.0)));
    ids.sort_unstable();
    assert_eq!(ids, [10, 20, 110, 120, 210]);
}

#[test]
fn update() {
    let mut application = Application::new()
        .with_group(build_group_to_replace)
        .with_group(build_group_to_delete)
        .with_group(build_group_with_added_entity);

    application.update();

    let mut ids = Vec::new();
    application.run(system_once!(|p: &Parent| ids.push(p.0)));
    ids.sort_unstable();
    assert_eq!(ids, [30, 40, 210, 220]);
}
