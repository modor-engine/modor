use approx::assert_abs_diff_eq;
use modor_physics::Position;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_2d_position() {
    let position = Position::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 0.);
    assert_abs_diff_eq!(position.abs().x, 1.);
    assert_abs_diff_eq!(position.abs().y, 2.);
    assert_abs_diff_eq!(position.abs().z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_3d_position() {
    let position = Position::xyz(1., 2., 3.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.z, 3.);
    assert_abs_diff_eq!(position.abs().x, 1.);
    assert_abs_diff_eq!(position.abs().y, 2.);
    assert_abs_diff_eq!(position.abs().z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_position() {
    let mut position1 = Position::xyz(1., 2., 3.);
    position1.x = 0.;
    let mut position2 = Position::xyz(4., 6., 8.);
    position2.x = 0.;
    assert_abs_diff_eq!(position1.distance(&position2), 50.0_f32.sqrt());
    assert_abs_diff_eq!(position2.distance(&position1), 50.0_f32.sqrt());
}
