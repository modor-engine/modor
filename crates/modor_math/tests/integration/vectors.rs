use approx::assert_abs_diff_eq;
use modor_math::{Vec2D, Vec3D};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector2d() {
    let vec1 = Vec2D::<()>::xy(1., 2.);
    assert_abs_diff_eq!(vec1.magnitude(), 5_f32.sqrt());
    let vec2 = vec1.with_magnitude(20_f32.sqrt()).unwrap();
    assert_abs_diff_eq!(vec2.x, 2.);
    assert_abs_diff_eq!(vec2.y, 4.);
    assert!(Vec2D::<()>::xy(0., 0.).with_magnitude(2.).is_none());
    let vec3 = vec2.with_unit::<fn()>();
    assert_abs_diff_eq!(vec3.x, 2.);
    assert_abs_diff_eq!(vec3.y, 4.);
    let vec4 = vec2.with_z(6.);
    assert_abs_diff_eq!(vec4.x, 2.);
    assert_abs_diff_eq!(vec4.y, 4.);
    assert_abs_diff_eq!(vec4.z, 6.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector3d() {
    let vec1 = Vec3D::<()>::xyz(1., 2., 3.);
    assert_abs_diff_eq!(vec1.magnitude(), 14_f32.sqrt());
    let vec2 = vec1.with_magnitude(56_f32.sqrt()).unwrap();
    assert_abs_diff_eq!(vec2.x, 2.);
    assert_abs_diff_eq!(vec2.y, 4.);
    assert_abs_diff_eq!(vec2.z, 6.);
    assert!(Vec3D::<()>::xyz(0., 0., 0.).with_magnitude(2.).is_none());
}
