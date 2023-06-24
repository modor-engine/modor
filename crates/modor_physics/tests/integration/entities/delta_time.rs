use modor::{App, With};
use modor_physics::{DeltaTime, PhysicsModule};
use std::time::Duration;

#[modor_test]
fn update_delta_time() {
    App::new()
        .with_entity(PhysicsModule::build())
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::ZERO))
        })
        .with_entity(DeltaTime::from(Duration::from_millis(5)))
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(5)))
        })
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_millis(100)))
        .assert::<With<DeltaTime>>(1, |e| {
            e.has(|t: &DeltaTime| assert_eq!(t.get(), Duration::from_millis(100)))
        });
}
