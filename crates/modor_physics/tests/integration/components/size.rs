use approx::assert_abs_diff_eq;
use modor_math::Vec3;
use modor_physics::{RelativeSize, Size};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_size() {
    let mut size = Size::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(size.x, 1.);
    assert_abs_diff_eq!(size.y, 2.);
    assert_abs_diff_eq!(size.z, 3.);
    size.x = 4.;
    assert_abs_diff_eq!(size.x, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_relative_size() {
    let mut size = RelativeSize::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(size.x, 1.);
    assert_abs_diff_eq!(size.y, 2.);
    assert_abs_diff_eq!(size.z, 3.);
    size.x = 4.;
    assert_abs_diff_eq!(size.x, 4.);
}
