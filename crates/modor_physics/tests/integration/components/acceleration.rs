use approx::assert_abs_diff_eq;
use modor_physics::Acceleration;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_acceleration() {
    let acceleration = Acceleration::xy(1., 2.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_acceleration() {
    let acceleration = Acceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_acceleration() {
    let mut acceleration = Acceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.magnitude(), 14.0_f32.sqrt());
    acceleration.set_magnitude(14.0_f32.sqrt() * 2.);
    assert_abs_diff_eq!(acceleration.x, 2.);
    assert_abs_diff_eq!(acceleration.y, 4.);
    assert_abs_diff_eq!(acceleration.z, 6.);
    acceleration.set_magnitude(0.);
    acceleration.set_magnitude(1.);
    assert_abs_diff_eq!(acceleration.x, 0.);
    assert_abs_diff_eq!(acceleration.y, 0.);
    assert_abs_diff_eq!(acceleration.z, 0.);
}
