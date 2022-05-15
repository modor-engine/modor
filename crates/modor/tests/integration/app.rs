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
fn create_app_with_thread_count() {
    let app = App::new().with_thread_count(2);
    assert_eq!(app.thread_count(), 2);
    let app = app.with_thread_count(1);
    assert_eq!(app.thread_count(), 1);
    let app = app.with_thread_count(0);
    assert_eq!(app.thread_count(), 1);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn run_for_singleton() {
    let mut app = App::new().with_entity(Singleton::build(10));
    app.run_for_singleton(|s: &mut Singleton| s.0 = 20);
    let app: TestApp = app.into();
    app.assert_singleton::<Singleton>()
        .has(|s: &Singleton| assert_eq!(s.0, 20));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn start_runner() {
    let mut run = false;
    App::new().run(|_| run = true);
    assert!(run);
}

#[allow(dead_code)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_app_with_wasm() {
    let app = App::new().with_thread_count(2);
    assert_eq!(app.thread_count(), 1);
}
