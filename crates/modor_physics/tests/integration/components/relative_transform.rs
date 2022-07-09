use approx::assert_abs_diff_eq;
use modor_math::{Quat, Vec3};
use modor_physics::RelativeTransform;
use std::f32::consts::PI;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_new_transform() {
    let body = RelativeTransform::new();
    assert!(body.position.is_none());
    assert!(body.size.is_none());
    assert!(body.rotation.is_none());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_customized_transform() {
    let body = RelativeTransform::new()
        .with_position(Vec3::xyz(1., 2., 3.))
        .with_size(Vec3::xyz(4., 5., 6.))
        .with_rotation(Quat::from_z(PI));
    assert_abs_diff_eq!(body.position.unwrap().x, 1.);
    assert_abs_diff_eq!(body.position.unwrap().y, 2.);
    assert_abs_diff_eq!(body.position.unwrap().z, 3.);
    assert_abs_diff_eq!(body.size.unwrap().x, 4.);
    assert_abs_diff_eq!(body.size.unwrap().y, 5.);
    assert_abs_diff_eq!(body.size.unwrap().z, 6.);
    assert_abs_diff_eq!(body.rotation.unwrap().angle(), PI);
}
