use approx::assert_abs_diff_eq;
use modor_math::{Vector2D, Vector3D};

#[derive(Clone, Copy)]
struct Movement2D(f32, f32);

impl Vector2D for Movement2D {
    fn create(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    fn components(self) -> (f32, f32) {
        (self.0, self.1)
    }
}

#[derive(Clone, Copy)]
struct Movement3D(f32, f32, f32);

impl Vector3D for Movement3D {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self(x, y, z)
    }

    fn components(self) -> (f32, f32, f32) {
        (self.0, self.1, self.2)
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector2d() {
    let point = Movement2D(1., 2.);
    assert_abs_diff_eq!(point.magnitude(), 5_f32.sqrt());
    let point = point.with_magnitude(20_f32.sqrt());
    assert_abs_diff_eq!(point.0, 2.);
    assert_abs_diff_eq!(point.1, 4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector3d() {
    let point = Movement3D(1., 2., 3.);
    assert_abs_diff_eq!(point.magnitude(), 14_f32.sqrt());
    let point = point.with_magnitude(56_f32.sqrt());
    assert_abs_diff_eq!(point.0, 2.);
    assert_abs_diff_eq!(point.1, 4.);
    assert_abs_diff_eq!(point.2, 6.);
}
