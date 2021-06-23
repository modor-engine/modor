use modor::*;

fn build_group(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<Child>((10, 20));
    builder.with_entity::<Parent>(30);
    builder.with_entity::<Child>((40, -1));
}

struct Parent(u32);

impl EntityMainComponent for Parent {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder
            .with::<String>(format!("Parent initial value: {}", data))
            .with_self(Self(data))
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::increment));
    }
}

impl Parent {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

struct Child(i64);

impl EntityMainComponent for Child {
    type Data = (u32, i64);

    fn build(builder: &mut EntityBuilder<'_, Self>, (parent, child): Self::Data) -> Built {
        builder
            .inherit_from::<Parent>(parent)
            .with::<String>(format!("Child initial value: {}", child))
            .with_self(Self(child))
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner
            .run(system!(Self::increment))
            .run(system!(Self::duplicate_if_not_zero));
    }
}

impl Child {
    fn increment(&mut self, message: &mut String) {
        self.0 += 1;
        *message = format!("Child value: {}", self.0);
    }

    fn duplicate_if_not_zero(&self, parent: &Parent, mut group: Group<'_>) {
        if self.0 != 0 {
            group.create_entity::<Self>((parent.0, self.0));
        }
    }
}

#[test]
fn init() {
    let mut application = Application::new().with_group(build_group);

    let mut components = Vec::new();
    application.run(system_once!(|p: &Parent, c: Option<&Child>, m: &String| {
        components.push((p.0, c.map(|p| p.0), m.clone()))
    }));
    components.sort_unstable();
    let expected_components = [
        (10, Some(20), "Child initial value: 20".into()),
        (30, None, "Parent initial value: 30".into()),
        (40, Some(-1), "Child initial value: -1".into()),
    ];
    assert_eq!(components, expected_components)
}

#[test]
fn update() {
    let mut application = Application::new().with_group(build_group);

    application.update();

    let mut components = Vec::new();
    application.run(system_once!(|p: &Parent, c: Option<&Child>, m: &String| {
        components.push((p.0, c.map(|p| p.0), m.clone()))
    }));
    components.sort_unstable();
    let expected_components = [
        (11, Some(21), "Child initial value: 21".into()),
        (11, Some(21), "Child value: 21".into()),
        (31, None, "Parent initial value: 30".into()),
        (41, Some(0), "Child value: 0".into()),
    ];
    assert_eq!(components, expected_components)
}
