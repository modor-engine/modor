use modor::*;

fn build_group_to_replace(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<GroupReplacer>(10);
    builder.with_entity::<GroupReplacer>(20);
}

struct GroupReplacer(u32);

impl EntityMainComponent for GroupReplacer {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, id: Self::Data) -> Built {
        builder.with_self(Self(id))
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::replace_group));
    }
}

impl GroupReplacer {
    fn replace_group(mut group: Group<'_>) {
        group.replace(|b| {
            b.with_entity::<Self>(100);
            b.with_entity::<Self>(200);
        });
    }
}

#[test]
fn init() {
    let mut application = Application::new().with_group(build_group_to_replace);

    let mut ids = Vec::new();
    application.run(system_once!(|r: &GroupReplacer| ids.push(r.0)));
    ids.sort_unstable();
    assert_eq!(ids, [10, 20]);
}

#[test]
fn update() {
    let mut application = Application::new().with_group(build_group_to_replace);

    application.update();

    let mut numbers = Vec::new();
    application.run(system_once!(|r: &GroupReplacer| numbers.push(r.0)));
    numbers.sort_unstable();
    assert_eq!(numbers, [100, 200]);
}
