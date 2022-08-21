use crate::collisions::CollisionGroup;
use modor::{Built, EntityBuilder};
use modor_math::{Quat, Vec3};
use modor_physics::{Collider, Transform};
use std::f32::consts::FRAC_PI_4;

struct Rectangle;

#[entity]
impl Rectangle {
    fn build(position: Vec3, size: Vec3, rotation: Quat) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform::new()
                    .with_position(position)
                    .with_size(size)
                    .with_rotation(rotation),
            )
            .with(Collider::rectangle_2d(CollisionGroup::MAIN))
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_one_shape_fully_in_other() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.15, 0.1, 0.), Vec3::ONE * 0.5, Quat::ZERO),
        Vec3::new(0.6, 0., 0.),
        Vec3::new(0.15, 0.1, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.15, 0.), Vec3::ONE * 0.5, Quat::ZERO),
        Vec3::new(0., 0.6, 0.),
        Vec3::new(0.1, 0.15, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.15, 0.), Vec3::ZERO, Quat::ZERO),
        Vec3::new(0., 0.35, 0.),
        Vec3::new(0.1, 0.15, 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_same_shape() {
    super::assert_collision(
        || Rectangle::build(Vec3::new(0.15, 0.1, 0.), Vec3::new(1., 2., 1.), Quat::ZERO),
        || {
            Rectangle::build(
                Vec3::new(0.15 + f32::EPSILON, 0.1, 0.), // epsilon used for determinism
                Vec3::new(1., 2., 2.),
                Quat::ZERO,
            )
        },
        Vec3::new(1., 0., 0.),
        Vec3::new(0.15, 0.1, 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_partial_collision() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::new(1., 2., 1.), Quat::ZERO),
        || Rectangle::build(Vec3::new(0.15, 0.1, 0.), Vec3::new(2., 1., 2.), Quat::ZERO),
        Vec3::new(1.35, 0., 0.),
        Vec3::new(0.0, 0.1, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.15, 0.1, 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0.85, 0., 0.),
        Vec3::new(0.075, 0.05, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1., 1., 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0., 0.),
        Vec3::new(0.5, 0.5, 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_no_collision() {
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1. + f32::EPSILON, 1., 0.), Vec3::ONE, Quat::ZERO),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1., 1. + f32::EPSILON, 0.), Vec3::ONE, Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_rotated() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::from_x(FRAC_PI_4)),
        || Rectangle::build(Vec3::new(0., -0.5, 0.), Vec3::new(2., 1., 0.), Quat::ZERO),
        Vec3::new(0., -FRAC_PI_4.cos() / 2., 0.),
        Vec3::new(0., 0., 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::from_y(FRAC_PI_4)),
        || Rectangle::build(Vec3::new(-0.5, 0., 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
        Vec3::new(-FRAC_PI_4.cos() / 2., 0., 0.),
        Vec3::new(0., 0., 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::from_z(FRAC_PI_4)),
        || Rectangle::build(Vec3::new(-0.5, 0., 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
        Vec3::new(-2_f32.sqrt() / 2., 0., 0.),
        Vec3::new(0., 0., 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1.1, 1.1, 0.), Vec3::ONE, Quat::from_z(FRAC_PI_4)),
    );
}
