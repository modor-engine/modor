use crate::TestEntity;
use modor::{App, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, DeltaTime, Dynamics2D, PhysicsModule, Transform2D,
};
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    let entity = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1., 2.)))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.position = Vec2::new(3., 4.))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 4.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_size() {
    let entity = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_size(Vec2::new(1., 2.)))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(1., 2.)))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.size = Vec2::new(3., 4.))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(3., 4.)))
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_rotation() {
    let entity = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_rotation(PI))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -PI))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.rotation = FRAC_PI_2)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_2))
        });
}
