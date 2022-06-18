use approx::assert_abs_diff_eq;
use modor_physics::{RelativeVelocity, Velocity};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_velocity() {
    let velocity = Velocity::xy(1., 2.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 0.);
    let mut velocity = Velocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    assert_abs_diff_eq!(velocity.x, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_relative_velocity() {
    let velocity = RelativeVelocity::xy(1., 2.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 0.);
    let mut velocity = RelativeVelocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
    velocity.x = 4.;
    assert_abs_diff_eq!(velocity.x, 4.);
}
