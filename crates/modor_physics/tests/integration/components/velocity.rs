use approx::assert_abs_diff_eq;
use modor_math::Vec3;
use modor_physics::{RelativeVelocity, Velocity};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_velocity() {
    let mut velocity = Velocity::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    let vec: Vec3 = velocity.into();
    assert_abs_diff_eq!(vec.x, 4.);
    assert_abs_diff_eq!(vec.y, 2.);
    assert_abs_diff_eq!(vec.z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_relative_velocity() {
    let mut velocity = RelativeVelocity::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    let vec: Vec3 = velocity.into();
    assert_abs_diff_eq!(vec.x, 4.);
    assert_abs_diff_eq!(vec.y, 2.);
    assert_abs_diff_eq!(vec.z, 3.);
}
