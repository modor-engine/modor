#[macro_use]
extern crate modor;

use modor::testing::TestApp;
use modor::{Built, EntityBuilder, SingleMut};

struct MainModule;

#[singleton]
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

struct Total(u32);

#[singleton]
impl Total {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0))
    }
}

struct Incrementer(u32);

#[entity]
impl Incrementer {
    fn build(increment: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(increment))
    }

    #[run]
    fn update(&self, mut total: SingleMut<'_, Total>) {
        total.0 += self.0;
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_singletons() {
    let mut app = TestApp::new();
    app.create_entity(MainModule::build(vec![5, 1, 2]));
    app.update();
    app.assert_singleton::<MainModule>().exists();
    app.assert_singleton::<Total>()
        .has::<Total, _>(|t| assert_eq!(t.0, 8));
}
