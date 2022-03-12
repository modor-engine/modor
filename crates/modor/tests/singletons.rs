use modor::testing::TestApp;
use modor::{
    system, Built, EntityBuilder, EntityMainComponent, SingleMut, Singleton, SystemRunner,
};

struct MainModule;

impl MainModule {
    fn build(increments: Vec<u32>) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_dependency(Total::build())
            .with_children(|b| {
                for increment in increments {
                    b.add(Incrementer::build(increment));
                }
            })
    }
}

impl EntityMainComponent for MainModule {
    type Type = Singleton;
}

struct Total(u32);

impl Total {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0))
    }
}

impl EntityMainComponent for Total {
    type Type = Singleton;
}

struct Incrementer(u32);

impl Incrementer {
    fn build(increment: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(increment))
    }

    fn update(&self, mut total: SingleMut<'_, Total>) {
        total.0 += self.0;
    }
}

impl EntityMainComponent for Incrementer {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::update))
    }
}

#[test]
fn update_singletons() {
    let mut app = TestApp::new();
    app.create_entity(MainModule::build(vec![5, 1, 2]));
    app.update();
    app.assert_singleton::<MainModule>().exists();
    app.assert_singleton::<Total>()
        .has::<Total, _>(|t| assert_eq!(t.0, 8));
}
