use modor::testing::TestApp;
use modor::{
    system, Built, EntityBuilder, EntityMainComponent, GlobMut, Global, GlobalBuilder, SystemRunner,
};

struct MainModule;

impl Global for MainModule {
    fn build(builder: GlobalBuilder<'_>) {
        builder
            .with_dependency(Total(0))
            .with_entity::<Incrementer>(2)
            .with_entity::<Incrementer>(1);
    }
}

struct Total(u32);

impl Global for Total {
    fn on_update(runner: SystemRunner<'_>) {
        runner.run(system!(Self::update));
    }
}

impl Total {
    fn update(mut self_: GlobMut<'_, Self>, incrementer: &Incrementer) {
        self_.0 += incrementer.0;
    }
}

struct Incrementer(u32);

impl EntityMainComponent for Incrementer {
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, increment: Self::Data) -> Built {
        builder.with_self(Self(increment))
    }
}

#[test]
fn update_globals() {
    let mut app = TestApp::new();
    app.create_global(MainModule);
    app.update();
    app.assert_global_exists::<MainModule, _>(|_| ());
    app.assert_global_exists::<Total, _>(|t| assert_eq!(t.0, 3));
}
