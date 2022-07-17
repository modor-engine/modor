use approx::assert_abs_diff_eq;
use modor_math::Vec2;
use std::f32::consts::FRAC_PI_2;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create() {
    let vec = Vec2::default();
    assert_abs_diff_eq!(vec.x, 0.);
    assert_abs_diff_eq!(vec.y, 0.);
    let vec = Vec2::new(1., 2.);
    assert_abs_diff_eq!(vec.x, 1.);
    assert_abs_diff_eq!(vec.y, 2.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_z() {
    let vec = Vec2::new(1., 2.).with_z(3.);
    assert_abs_diff_eq!(vec.x, 1.);
    assert_abs_diff_eq!(vec.y, 2.);
    assert_abs_diff_eq!(vec.z, 3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_scale() {
    let vec = Vec2::new(1., 2.).with_scale(Vec2::new(5., 3.));
    assert_abs_diff_eq!(vec.x, 5.);
    assert_abs_diff_eq!(vec.y, 6.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_magnitude() {
    let vec = Vec2::new(1., 2.).with_magnitude(20_f32.sqrt()).unwrap();
    assert_abs_diff_eq!(vec.x, 2.);
    assert_abs_diff_eq!(vec.y, 4.);
    assert!(Vec2::new(0., 0.).with_magnitude(2.).is_none());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_rotation_between_2_vecs() {
    let rotation = Vec2::new(0.5, 0.5).rotation(Vec2::new(0.5, -0.5));
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    assert_abs_diff_eq!(rotation.axis().unwrap().x, 0.);
    assert_abs_diff_eq!(rotation.axis().unwrap().y, 0.);
    assert_abs_diff_eq!(rotation.axis().unwrap().z, -1.);
    let rotation = Vec2::new(0.5, -0.5).rotation(Vec2::new(0.5, 0.5));
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    assert_abs_diff_eq!(rotation.axis().unwrap().x, 0.);
    assert_abs_diff_eq!(rotation.axis().unwrap().y, 0.);
    assert_abs_diff_eq!(rotation.axis().unwrap().z, 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_dot_product() {
    let dot = Vec2::new(1., 2.).dot(Vec2::new(3., 4.));
    assert_abs_diff_eq!(dot, 11.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_mirror_vec() {
    let mirror = Vec2::new(0.7, 0.3).mirror(Vec2::new(2., 2.));
    assert_abs_diff_eq!(mirror.x, 0.3);
    assert_abs_diff_eq!(mirror.y, 0.7);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_vec() {
    let new_vec = Vec2::new(1., 2.) + Vec2::new(3., 5.);
    assert_abs_diff_eq!(new_vec.x, 4.);
    assert_abs_diff_eq!(new_vec.y, 7.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec += Vec2::new(3., 5.);
    assert_abs_diff_eq!(new_vec.x, 4.);
    assert_abs_diff_eq!(new_vec.y, 7.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn sub_vec() {
    let new_vec = Vec2::new(1., 2.) - Vec2::new(3., 5.);
    assert_abs_diff_eq!(new_vec.x, -2.);
    assert_abs_diff_eq!(new_vec.y, -3.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec -= Vec2::new(3., 5.);
    assert_abs_diff_eq!(new_vec.x, -2.);
    assert_abs_diff_eq!(new_vec.y, -3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn mul_float() {
    let new_vec = Vec2::new(1., 2.) * 5.;
    assert_abs_diff_eq!(new_vec.x, 5.);
    assert_abs_diff_eq!(new_vec.y, 10.);
    let new_vec = 5. * Vec2::new(1., 2.);
    assert_abs_diff_eq!(new_vec.x, 5.);
    assert_abs_diff_eq!(new_vec.y, 10.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec *= 5.;
    assert_abs_diff_eq!(new_vec.x, 5.);
    assert_abs_diff_eq!(new_vec.y, 10.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn div_float() {
    let new_vec = Vec2::new(1., 2.) / 5.;
    assert_abs_diff_eq!(new_vec.x, 0.2);
    assert_abs_diff_eq!(new_vec.y, 0.4);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec /= 5.;
    assert_abs_diff_eq!(new_vec.x, 0.2);
    assert_abs_diff_eq!(new_vec.y, 0.4);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector() {
    let vec = Vec2::new(1., 2.);
    assert_abs_diff_eq!(vec.magnitude(), 5_f32.sqrt());
    assert_abs_diff_eq!(vec.distance(Vec2::new(4., 3.)), 10_f32.sqrt());
}
