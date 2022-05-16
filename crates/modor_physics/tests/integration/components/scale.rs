use approx::assert_abs_diff_eq;
use modor_physics::Scale;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_scale() {
    let scale = Scale::xy(4., 2.);
    assert_abs_diff_eq!(scale.x, 4.);
    assert_abs_diff_eq!(scale.y, 2.);
    assert_abs_diff_eq!(scale.z, 1.);
    assert_abs_diff_eq!(scale.abs().x, 4.);
    assert_abs_diff_eq!(scale.abs().y, 2.);
    assert_abs_diff_eq!(scale.abs().z, 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_scale() {
    let scale = Scale::xyz(1., 2., 3.);
    assert_abs_diff_eq!(scale.x, 1.);
    assert_abs_diff_eq!(scale.y, 2.);
    assert_abs_diff_eq!(scale.z, 3.);
    assert_abs_diff_eq!(scale.abs().x, 1.);
    assert_abs_diff_eq!(scale.abs().y, 2.);
    assert_abs_diff_eq!(scale.abs().z, 3.);
}
