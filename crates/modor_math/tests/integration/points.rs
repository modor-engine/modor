use approx::assert_abs_diff_eq;
use modor_math::{Point2D, Point3D};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_point2d() {
    let point1 = Point2D::<()>::xy(1., 2.);
    let point2 = Point2D::<()>::xy(3., 5.);
    assert_abs_diff_eq!(point1.distance(point2), 13_f32.sqrt());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_point3d() {
    let point1 = Point3D::<()>::xyz(1., 2., 3.);
    let point2 = Point3D::<()>::xyz(3., 5., 7.);
    assert_abs_diff_eq!(point1.distance(point2), 29_f32.sqrt());
}
