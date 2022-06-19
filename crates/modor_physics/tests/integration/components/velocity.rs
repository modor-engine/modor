use approx::assert_abs_diff_eq;
use modor_math::Vec3;
use modor_physics::{RelativeVelocity, Velocity};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_velocity() {
    let mut velocity = Velocity::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    assert_abs_diff_eq!(velocity.x, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_relative_velocity() {
    let mut velocity = RelativeVelocity::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    assert_abs_diff_eq!(velocity.x, 4.);
}
