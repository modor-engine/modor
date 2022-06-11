use approx::assert_abs_diff_eq;
use modor_math::Point3D;
use modor_physics::{Position, RelativePosition};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_position() {
    let position = Position::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_position() {
    let position = Position::xyz(1., 2., 3.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    assert_abs_diff_eq!(position.components().0, 1.);
    assert_abs_diff_eq!(position.components().1, 2.);
    assert_abs_diff_eq!(position.components().2, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_relative_position() {
    let position = RelativePosition::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_relative_position() {
    let position = RelativePosition::xyz(1., 2., 3.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    assert_abs_diff_eq!(position.components().0, 1.);
    assert_abs_diff_eq!(position.components().1, 2.);
    assert_abs_diff_eq!(position.components().2, 3.);
}
