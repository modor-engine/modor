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
fn check_with_x_offset() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0., 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0.9, 0., 0.),
        Vec3::new(0.05, 0., 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1., 0., 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0., 0.),
        Vec3::new(0.5, 0., 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1.1, 0., 0.), Vec3::ONE, Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_y_offset() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0., 0.1, 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0.9, 0.),
        Vec3::new(0., 0.05, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0., 1., 0.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0., 0.),
        Vec3::new(0., 0.5, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0., 1.1, 0.), Vec3::ONE, Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_z_offset() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.01, 0., 0.1), Vec3::ONE, Quat::ZERO),
        Vec3::new(0.99, 0., 0.),
        Vec3::new(0.005, 0., 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_offset_on_multiple_axes() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.11, 0.1), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0.89, 0.),
        Vec3::new(0.05, 0.055, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1., 1., 1.), Vec3::ONE, Quat::ZERO),
        Vec3::new(0., 0., 0.),
        Vec3::new(0.5, 0.5, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1.1, 1.1, 1.1), Vec3::ONE, Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_different_sizes() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
        Vec3::new(0.9, 0., 0.),
        Vec3::new(-0.4, 0., 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1.1, 1.6, 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_zero_x_size() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::new(0., 1., 1.), Quat::ZERO),
        Vec3::new(0.4, 0., 0.),
        Vec3::new(0.1, 0.5, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 1.01, 0.), Vec3::new(0., 1., 1.), Quat::ZERO),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.51, 0.1, 0.), Vec3::new(0., 1., 1.), Quat::ZERO),
    );
}
#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_zero_y_size() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::new(1., 0., 1.), Quat::ZERO),
        Vec3::new(0., 0.4, 0.),
        Vec3::new(0.5, 0.1, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(1.01, 0.1, 0.), Vec3::new(1., 0., 1.), Quat::ZERO),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.51, 0.), Vec3::new(1., 0., 1.), Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_zero_z_size() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.11, 0.), Vec3::new(1., 1., 0.), Quat::ZERO),
        Vec3::new(0., 0.89, 0.),
        Vec3::new(0.05, 0.055, 0.),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.11, 1.), Vec3::new(1., 1., 0.), Quat::ZERO),
        Vec3::new(0., 0.89, 0.),
        Vec3::new(0.05, 0.055, 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_zero_size() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::ZERO, Quat::ZERO),
        Vec3::new(0.4, 0., 0.),
        Vec3::new(0.1, 0.1, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.51, 0.), Vec3::ZERO, Quat::ZERO),
    );
    super::assert_collision(
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::ZERO, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::ZERO, Quat::ZERO),
        Vec3::new(0., 0., 0.),
        Vec3::new(0.1, 0.1, 0.),
    );
    super::assert_no_collision(
        || Rectangle::build(Vec3::new(0.1, 0.1, 0.), Vec3::ZERO, Quat::ZERO),
        || Rectangle::build(Vec3::new(0.1, 0.2, 0.), Vec3::ZERO, Quat::ZERO),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_different_x_rotations() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::from_x(FRAC_PI_4)),
        || Rectangle::build(Vec3::new(-0.5, 0., 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
        Vec3::new(-0.5, 0., 0.),
        Vec3::new(0., 0., 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_different_y_rotations() {
    super::assert_collision(
        || Rectangle::build(Vec3::ZERO, Vec3::ONE, Quat::from_y(FRAC_PI_4)),
        || Rectangle::build(Vec3::new(-0.5, 0., 0.), Vec3::new(1., 2., 0.), Quat::ZERO),
        Vec3::new(-FRAC_PI_4.cos() / 2., 0., 0.),
        Vec3::new(0., 0., 0.),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_with_different_z_rotations() {
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
