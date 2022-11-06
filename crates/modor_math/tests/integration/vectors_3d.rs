use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use modor_math::Vec3;
use std::{f32::consts::FRAC_PI_2, iter};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create() {
    let vec = Vec3::default();
    assert_approx_eq!(vec.x, 0.);
    assert_approx_eq!(vec.y, 0.);
    let vec = Vec3::new(1., 2., 3.);
    assert_approx_eq!(vec.x, 1.);
    assert_approx_eq!(vec.y, 2.);
    assert_approx_eq!(vec.z, 3.);
    let vec = Vec3::from_xy(1., 2.);
    assert_approx_eq!(vec.x, 1.);
    assert_approx_eq!(vec.y, 2.);
    assert_approx_eq!(vec.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_scale() {
    let vec = Vec3::new(1., 2., 3.).with_scale(Vec3::new(5., 3., 4.));
    assert_approx_eq!(vec.x, 5.);
    assert_approx_eq!(vec.y, 6.);
    assert_approx_eq!(vec.z, 12.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_magnitude() {
    let vec = Vec3::new(1., 2., 3.).with_magnitude(56_f32.sqrt()).unwrap();
    assert_approx_eq!(vec.x, 2.);
    assert_approx_eq!(vec.y, 4.);
    assert_approx_eq!(vec.z, 6.);
    assert!(Vec3::new(0., 0., 0.).with_magnitude(2.).is_none());
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_rotation_between_2_vecs() {
    let rotation = Vec3::new(0.5, 0.5, 0.).rotation(Vec3::new(0.5, -0.5, 0.));
    assert_approx_eq!(rotation.angle(), FRAC_PI_2);
    assert_approx_eq!(rotation.axis().unwrap().x, 0.);
    assert_approx_eq!(rotation.axis().unwrap().y, 0.);
    assert_approx_eq!(rotation.axis().unwrap().z, -1.);
    let rotation = Vec3::new(0.5, -0.5, 0.).rotation(Vec3::new(0.5, 0.5, 0.));
    assert_approx_eq!(rotation.angle(), FRAC_PI_2);
    assert_approx_eq!(rotation.axis().unwrap().x, 0.);
    assert_approx_eq!(rotation.axis().unwrap().y, 0.);
    assert_approx_eq!(rotation.axis().unwrap().z, 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_dot_product() {
    let dot = Vec3::new(1., 2., 3.).dot(Vec3::new(4., 5., 6.));
    assert_approx_eq!(dot, 32.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_cross_product() {
    let cross = Vec3::new(1., 2., 3.).cross(Vec3::new(4., 5., 6.));
    assert_approx_eq!(cross.x, -3.);
    assert_approx_eq!(cross.y, 6.);
    assert_approx_eq!(cross.z, -3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn calculate_mirror_vec() {
    let mirror = Vec3::new(0.7, 0.3, 0.).mirror(Vec3::new(2., 2., 0.));
    assert_approx_eq!(mirror.x, 0.3);
    assert_approx_eq!(mirror.y, 0.7);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn add_vec() {
    let new_vec = Vec3::new(1., 2., 3.) + Vec3::new(3., 5., 7.);
    assert_approx_eq!(new_vec.x, 4.);
    assert_approx_eq!(new_vec.y, 7.);
    assert_approx_eq!(new_vec.z, 10.);
    let mut new_vec = Vec3::new(1., 2., 3.);
    new_vec += Vec3::new(3., 5., 7.);
    assert_approx_eq!(new_vec.x, 4.);
    assert_approx_eq!(new_vec.y, 7.);
    assert_approx_eq!(new_vec.z, 10.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn sub_vec() {
    let new_vec = Vec3::new(1., 2., 3.) - Vec3::new(3., 5., 7.);
    assert_approx_eq!(new_vec.x, -2.);
    assert_approx_eq!(new_vec.y, -3.);
    assert_approx_eq!(new_vec.z, -4.);
    let mut new_vec = Vec3::new(1., 2., 3.);
    new_vec -= Vec3::new(3., 5., 7.);
    assert_approx_eq!(new_vec.x, -2.);
    assert_approx_eq!(new_vec.y, -3.);
    assert_approx_eq!(new_vec.z, -4.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn mul_float() {
    let new_vec = Vec3::new(1., 2., 3.) * 5.;
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    assert_approx_eq!(new_vec.z, 15.);
    let new_vec = 5. * Vec3::new(1., 2., 3.);
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    assert_approx_eq!(new_vec.z, 15.);
    let mut new_vec = Vec3::new(1., 2., 3.);
    new_vec *= 5.;
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    assert_approx_eq!(new_vec.z, 15.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn div_float() {
    let new_vec = Vec3::new(1., 2., 3.) / 5.;
    assert_approx_eq!(new_vec.x, 0.2);
    assert_approx_eq!(new_vec.y, 0.4);
    assert_approx_eq!(new_vec.z, 0.6);
    let mut new_vec = Vec3::new(1., 2., 3.);
    new_vec /= 5.;
    assert_approx_eq!(new_vec.x, 0.2);
    assert_approx_eq!(new_vec.y, 0.4);
    assert_approx_eq!(new_vec.z, 0.6);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn neg_vec() {
    let new_vec = -Vec3::new(1., 2., 3.);
    assert_approx_eq!(new_vec.x, -1.);
    assert_approx_eq!(new_vec.y, -2.);
    assert_approx_eq!(new_vec.z, -3.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn sum_vecs() {
    let sum: Vec3 = [
        Vec3::new(1., 2., 3.),
        Vec3::new(4., 5., 6.),
        Vec3::new(7., 8., 9.),
    ]
    .into_iter()
    .sum();
    assert_approx_eq!(sum.x, 12.);
    assert_approx_eq!(sum.y, 15.);
    assert_approx_eq!(sum.z, 18.);
    let sum: Vec3 = iter::empty().sum();
    assert_approx_eq!(sum.x, 0.);
    assert_approx_eq!(sum.y, 0.);
    assert_approx_eq!(sum.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn abs_diff_eq() {
    assert!(Vec3::new(1., 2., 3.).abs_diff_eq(&Vec3::new(1., 2., 3.), f32::EPSILON));
    assert!(Vec3::new(1., 2., 3.).abs_diff_eq(&Vec3::new(1. + f32::EPSILON, 2., 3.), f32::EPSILON));
    assert!(Vec3::new(1., 2., 3.).abs_diff_eq(&Vec3::new(1., 2. + f32::EPSILON, 3.), f32::EPSILON));
    assert!(Vec3::new(1., 2., 3.).abs_diff_eq(&Vec3::new(1., 2., 3.), f32::EPSILON));
    assert!(!Vec3::new(1., 2., 3.)
        .abs_diff_eq(&Vec3::new(1. + 2. * f32::EPSILON, 2., 3.), f32::EPSILON));
    assert!(!Vec3::new(1., 2., 3.)
        .abs_diff_eq(&Vec3::new(1., 2. + 2. * f32::EPSILON, 3.), f32::EPSILON));
    assert_approx_eq!(Vec3::default_epsilon(), f32::EPSILON);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn relative_eq() {
    assert!(Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(1., 2., 3.), f32::EPSILON, 0.1));
    assert!(Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(0.91, 2., 3.), f32::EPSILON, 0.1));
    assert!(Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(1., 1.81, 3.), f32::EPSILON, 0.1));
    assert!(Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(1., 2., 2.71), f32::EPSILON, 0.1));
    assert!(!Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(0.9, 2., 3.), f32::EPSILON, 0.1));
    assert!(!Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(1., 1.8, 3.), f32::EPSILON, 0.1));
    assert!(!Vec3::new(1., 2., 3.).relative_eq(&Vec3::new(1., 2., 2.69), f32::EPSILON, 0.1));
    assert_approx_eq!(Vec3::default_max_relative(), f32::EPSILON);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn ulps_eq() {
    assert!(Vec3::new(1., 2., 3.).ulps_eq(&Vec3::new(1., 2., 3.), f32::EPSILON, 1));
    assert!(Vec3::new(1., 2., 3.).ulps_eq(&Vec3::new(1. + f32::EPSILON, 2., 3.), f32::EPSILON, 1));
    assert!(Vec3::new(1., 2., 3.).ulps_eq(
        &Vec3::new(1., 2. + 2. * f32::EPSILON, 3.),
        f32::EPSILON,
        1
    ));
    assert!(Vec3::new(1., 2., 3.).ulps_eq(
        &Vec3::new(1., 2., 3. + 2. * f32::EPSILON),
        f32::EPSILON,
        1
    ));
    assert!(!Vec3::new(1., 2., 3.).ulps_eq(
        &Vec3::new(1. + 2. * f32::EPSILON, 2., 3.),
        f32::EPSILON,
        1
    ));
    assert!(!Vec3::new(1., 2., 3.).ulps_eq(
        &Vec3::new(1., 2. + 3. * f32::EPSILON, 3.),
        f32::EPSILON,
        1
    ));
    assert!(!Vec3::new(1., 2., 3.).ulps_eq(
        &Vec3::new(1., 2., 3. + 3. * f32::EPSILON),
        f32::EPSILON,
        1
    ));
    assert_eq!(Vec3::default_max_ulps(), 4);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_vector() {
    let vec = Vec3::new(1., 2., 3.);
    assert_approx_eq!(vec.magnitude(), 14_f32.sqrt());
    assert_approx_eq!(vec.distance(Vec3::new(4., 3., 2.)), 11_f32.sqrt());
}
