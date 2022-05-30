use approx::assert_abs_diff_eq;
use modor_physics::RelativeSize;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_size() {
    let size = RelativeSize::xy(4., 2.);
    assert_abs_diff_eq!(size.x, 4.);
    assert_abs_diff_eq!(size.y, 2.);
    assert_abs_diff_eq!(size.z, 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_size() {
    let size = RelativeSize::xyz(1., 2., 3.);
    assert_abs_diff_eq!(size.x, 1.);
    assert_abs_diff_eq!(size.y, 2.);
    assert_abs_diff_eq!(size.z, 3.);
}
