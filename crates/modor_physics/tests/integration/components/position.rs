use approx::assert_abs_diff_eq;
use modor_physics::{Position, RelativePosition};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_position() {
    let position = Position::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 0.);
    let mut position = Position::xyz(1., 2., 3.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    position.x = 4.;
    assert_abs_diff_eq!(position.x, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_relative_position() {
    let position = RelativePosition::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 0.);
    let mut position = RelativePosition::xyz(1., 2., 3.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    position.x = 4.;
    assert_abs_diff_eq!(position.x, 4.);
}
