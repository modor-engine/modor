use approx::assert_abs_diff_eq;
use modor_math::{Quat, Vec3};
use modor_physics::Transform;
use std::f32::consts::PI;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_default_transform() {
    let body = Transform::default();
    assert_abs_diff_eq!(body.position.x, 0.);
    assert_abs_diff_eq!(body.position.y, 0.);
    assert_abs_diff_eq!(body.position.z, 0.);
    assert_abs_diff_eq!(body.size.x, 1.);
    assert_abs_diff_eq!(body.size.y, 1.);
    assert_abs_diff_eq!(body.size.z, 1.);
    assert_abs_diff_eq!(body.rotation.angle(), 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_new_transform() {
    let body = Transform::new();
    assert_abs_diff_eq!(body.position.x, 0.);
    assert_abs_diff_eq!(body.position.y, 0.);
    assert_abs_diff_eq!(body.position.z, 0.);
    assert_abs_diff_eq!(body.size.x, 1.);
    assert_abs_diff_eq!(body.size.y, 1.);
    assert_abs_diff_eq!(body.size.z, 1.);
    assert_abs_diff_eq!(body.rotation.angle(), 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_customized_transform() {
    let body = Transform::new()
        .with_position(Vec3::new(1., 2., 3.))
        .with_size(Vec3::new(4., 5., 6.))
        .with_rotation(Quat::from_z(PI));
    assert_abs_diff_eq!(body.position.x, 1.);
    assert_abs_diff_eq!(body.position.y, 2.);
    assert_abs_diff_eq!(body.position.z, 3.);
    assert_abs_diff_eq!(body.size.x, 4.);
    assert_abs_diff_eq!(body.size.y, 5.);
    assert_abs_diff_eq!(body.size.z, 6.);
    assert_abs_diff_eq!(body.rotation.angle(), PI);
}
