use modor::{App, With};
use modor_physics::{DeltaTime, PhysicsModule};
use std::time::Duration;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_delta_time() {
    App::new()
        .with_entity(PhysicsModule::build())
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::ZERO))
        })
        .with_entity(DeltaTime::build(Duration::from_millis(5)))
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(5)))
        })
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_millis(10)))
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(10)))
        });
}
