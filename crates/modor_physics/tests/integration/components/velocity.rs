use approx::assert_abs_diff_eq;
use modor_physics::Velocity;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_velocity() {
    let velocity = Velocity::xy(1., 2.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_velocity() {
    let velocity = Velocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.x, 1.);
    assert_abs_diff_eq!(velocity.y, 2.);
    assert_abs_diff_eq!(velocity.z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_velocity() {
    let mut velocity = Velocity::xyz(1., 2., 3.);
    assert_abs_diff_eq!(velocity.magnitude(), 14.0_f32.sqrt());
    velocity.set_magnitude(14.0_f32.sqrt() * 2.);
    assert_abs_diff_eq!(velocity.x, 2.);
    assert_abs_diff_eq!(velocity.y, 4.);
    assert_abs_diff_eq!(velocity.z, 6.);
    velocity.set_magnitude(0.);
    velocity.set_magnitude(1.);
    assert_abs_diff_eq!(velocity.x, 0.);
    assert_abs_diff_eq!(velocity.y, 0.);
    assert_abs_diff_eq!(velocity.z, 0.);
}
