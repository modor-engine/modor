use approx::assert_abs_diff_eq;
use modor_input::{InputDelta, WindowPosition};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_input_delta() {
    let mut delta = InputDelta::xy(1., 2.);
    assert_abs_diff_eq!(delta.x, 1.);
    assert_abs_diff_eq!(delta.y, 2.);
    delta.x = 3.;
    assert_abs_diff_eq!(delta.x, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_window_position() {
    let mut position = WindowPosition::xy(1., 2.);
    assert_abs_diff_eq!(position.x, 1.);
    assert_abs_diff_eq!(position.y, 2.);
    position.x = 3.;
    assert_abs_diff_eq!(position.x, 3.);
}
