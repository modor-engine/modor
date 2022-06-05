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
