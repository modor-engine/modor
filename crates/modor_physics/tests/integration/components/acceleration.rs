use approx::assert_abs_diff_eq;
use modor_physics::{Acceleration, RelativeAcceleration};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_acceleration() {
    let acceleration = Acceleration::xy(1., 2.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 0.);
    let mut acceleration = Acceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 3.);
    acceleration.x = 4.;
    assert_abs_diff_eq!(acceleration.x, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_relative_acceleration() {
    let acceleration = RelativeAcceleration::xy(1., 2.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 0.);
    let mut acceleration = RelativeAcceleration::xyz(1., 2., 3.);
    assert_abs_diff_eq!(acceleration.x, 1.);
    assert_abs_diff_eq!(acceleration.y, 2.);
    assert_abs_diff_eq!(acceleration.z, 3.);
    acceleration.x = 4.;
    assert_abs_diff_eq!(acceleration.x, 4.);
}
