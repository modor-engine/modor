use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};

struct Singleton(u32);

#[singleton]
impl Singleton {
    fn build(value: u32) -> impl Built<Self> {
        EntityBuilder::new(Self(value))
    }
}

#[test]
fn create_app() {
    let mut app = App::new()
        .with_thread_count(2)
        .with_entity(Singleton::build(10));
    app.run_for_singleton(|s: &mut Singleton| s.0 = 20);
    let app: TestApp = app.into();
    assert_eq!(app.thread_count(), 2);
    app.assert_singleton::<Singleton>()
        .has(|s: &Singleton| assert_eq!(s.0, 20));
    let app: App = app.into();
    let mut run = false;
    app.run(|_| run = true);
    assert!(run);
}

#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_app_with_wasm() {
    let app = App::new().with_thread_count(2);
    assert_eq!(app.thread_count(), 1);
}
