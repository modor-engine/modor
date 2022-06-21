use approx::assert_abs_diff_eq;
use modor_math::Vec3;
use modor_physics::{Position, RelativePosition};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_position() {
    let mut position = Position::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    position.x = 4.;
    let vec: Vec3 = position.into();
    assert_abs_diff_eq!(vec.x, 4.);
    assert_abs_diff_eq!(vec.y, 2.);
    assert_abs_diff_eq!(vec.z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_relative_position() {
    let mut position = RelativePosition::from(Vec3::xyz(1., 2., 3.));
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    position.x = 4.;
    let vec: Vec3 = position.into();
    assert_abs_diff_eq!(vec.x, 4.);
    assert_abs_diff_eq!(vec.y, 2.);
    assert_abs_diff_eq!(vec.z, 3.);
}
