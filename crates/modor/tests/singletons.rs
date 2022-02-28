use modor::testing::TestApp;
use modor::{
    system, Built, EntityBuilder, EntityMainComponent, SingleMut, Singleton, SystemRunner,
};

struct MainModule;

impl EntityMainComponent for MainModule {
    type Type = Singleton;
    type Data = Vec<u32>;

    fn build(mut builder: EntityBuilder<'_, Self>, increments: Self::Data) -> Built<'_> {
        for increment in increments {
            builder = builder.with_child::<Incrementer>(increment);
        }
        builder.with_self(Self).with_dependency::<Total>(())
    }
}

struct Total(u32);

impl EntityMainComponent for Total {
    type Type = Singleton;
    type Data = ();

    fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built<'_> {
        builder.with_self(Self(0))
    }
}

struct Incrementer(u32);

impl EntityMainComponent for Incrementer {
    type Type = ();
    type Data = u32;

    fn build(builder: EntityBuilder<'_, Self>, increment: Self::Data) -> Built<'_> {
        builder.with_self(Self(increment))
    }

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run(system!(Self::update))
    }
}

impl Incrementer {
    fn update(&self, mut total: SingleMut<'_, Total>) {
        total.0 += self.0;
    }
}

#[test]
fn update_singletons() {
    let mut app = TestApp::new();
    app.create_entity::<MainModule>(vec![5, 1, 2]);
    app.update();
    app.assert_singleton::<MainModule>().exists();
    app.assert_singleton::<Total>()
        .has::<Total, _>(|t| assert_eq!(t.0, 8));
}
