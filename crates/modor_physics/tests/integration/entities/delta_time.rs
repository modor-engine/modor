use modor::testing::TestApp;
use modor::App;
use modor_physics::{DeltaTime, PhysicsModule};
use std::time::Duration;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_delta_time() {
    let app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    app.assert_singleton::<DeltaTime>()
        .has(|t: &DeltaTime| assert_eq!(t.get(), Duration::ZERO));

    let mut app: TestApp = App::from(app)
        .with_entity(DeltaTime::build(Duration::from_millis(5)))
        .into();
    app.assert_singleton::<DeltaTime>()
        .has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(5)));

    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_millis(10)));
    app.assert_singleton::<DeltaTime>()
        .has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(10)));
}
