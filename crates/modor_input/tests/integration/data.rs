use approx::assert_abs_diff_eq;
use modor_input::{InputDelta, WindowPosition};
use modor_math::{Point2D, Vector2D};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_input_delta() {
    let delta = InputDelta::xy(1., 2.);
    assert_abs_diff_eq!(delta.x, 1.);
    assert_abs_diff_eq!(delta.y, 2.);
    assert_abs_diff_eq!(delta.components().0, 1.);
    assert_abs_diff_eq!(delta.components().1, 2.);
    let delta = InputDelta::create(4., 5.);
    assert_abs_diff_eq!(delta.x, 4.);
    assert_abs_diff_eq!(delta.y, 5.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_window_position() {
    let position = WindowPosition::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    assert_abs_diff_eq!(position.components().0, 1.);
    assert_abs_diff_eq!(position.components().1, 2.);
}
