use approx::assert_abs_diff_eq;
use modor_math::{Vector2D, Vector3D};

#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign)]
struct Movement2D(f32, f32);

impl Vector2D for Movement2D {
    fn create(x: f32, y: f32) -> Self {
        Self(x, y)
    }

    fn components(self) -> (f32, f32) {
        (self.0, self.1)
    }
}

#[derive(Clone, Copy, Add, Sub, AddAssign, SubAssign)]
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
    let point1 = Movement2D(1., 2.);
    assert_abs_diff_eq!(point1.magnitude(), 5_f32.sqrt());
    let point2 = point1.with_magnitude(20_f32.sqrt());
    assert_abs_diff_eq!(point2.0, 2.);
    assert_abs_diff_eq!(point2.1, 4.);
    assert!(!point2.is_zero());
    let point3 = Movement2D(0., 0.).with_magnitude(2.);
    assert_abs_diff_eq!(point3.0, 0.);
    assert_abs_diff_eq!(point3.1, 0.);
    assert!(point3.is_zero());
    let point4 = point2.into_vec2::<Movement2D>();
    assert_abs_diff_eq!(point4.0, 2.);
    assert_abs_diff_eq!(point4.1, 4.);
    let point5 = point2.into_vec3::<Movement3D>();
    assert_abs_diff_eq!(point5.0, 2.);
    assert_abs_diff_eq!(point5.1, 4.);
    assert_abs_diff_eq!(point5.2, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector3d() {
    let point1 = Movement3D(1., 2., 3.);
    assert_abs_diff_eq!(point1.magnitude(), 14_f32.sqrt());
    let point2 = point1.with_magnitude(56_f32.sqrt());
    assert_abs_diff_eq!(point2.0, 2.);
    assert_abs_diff_eq!(point2.1, 4.);
    assert_abs_diff_eq!(point2.2, 6.);
    assert!(!point2.is_zero());
    let point3 = Movement3D(0., 0., 0.).with_magnitude(2.);
    assert_abs_diff_eq!(point3.0, 0.);
    assert_abs_diff_eq!(point3.1, 0.);
    assert_abs_diff_eq!(point3.2, 0.);
    assert!(point3.is_zero());
    let point5 = point2.into_vec3::<Movement3D>();
    assert_abs_diff_eq!(point5.0, 2.);
    assert_abs_diff_eq!(point5.1, 4.);
    assert_abs_diff_eq!(point5.2, 6.);
}
