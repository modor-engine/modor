use modor::*;

fn build_group(builder: &mut GroupBuilder<'_>) {
    builder.with_entity::<Number>(10);
    builder.with_entity::<Number>(20);
}

struct Number(u32);

impl EntityMainComponent for Number {
    type Data = u32;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.with_self(Self(data))
    }
}

#[test]
fn init() {
    let mut application = Application::new().with_group(build_group);

    let mut numbers = Vec::new();
    application.run(system_once!(|n: &Number| numbers.push(n.0)));
    numbers.sort_unstable();
    assert_eq!(numbers, [10, 20]);
}

#[test]
fn update() {
    let mut application = Application::new().with_group(build_group);

    application.update();

    let mut numbers = Vec::new();
    application.run(system_once!(|n: &Number| numbers.push(n.0)));
    numbers.sort_unstable();
    assert_eq!(numbers, [10, 20]);
}
