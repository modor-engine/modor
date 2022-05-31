use approx::assert_abs_diff_eq;
use modor_math::{Point2D, Point3D};

#[derive(Clone, Copy)]
struct Position2D(f32, f32);

impl Point2D for Position2D {
    type Unit = ();

    fn components(self) -> (f32, f32) {
        (self.0, self.1)
    }
}

#[derive(Clone, Copy)]
struct Position3D(f32, f32, f32);

impl Point3D for Position3D {
    type Unit = ();

    fn components(self) -> (f32, f32, f32) {
        (self.0, self.1, self.2)
    }
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_point2d() {
    let point1 = Position2D(1., 2.);
    let point2 = Position2D(3., 5.);
    assert_abs_diff_eq!(point1.distance(point2), 13_f32.sqrt());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_point3d() {
    let point1 = Position3D(1., 2., 3.);
    let point2 = Position3D(3., 5., 7.);
    assert_abs_diff_eq!(point1.distance(point2), 29_f32.sqrt());
}
