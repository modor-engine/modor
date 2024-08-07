use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use std::{f32::consts::FRAC_PI_2, iter};

#[modor::test]
fn create() {
    let vec = Vec2::default();
    assert_approx_eq!(vec.x, 0.);
    assert_approx_eq!(vec.y, 0.);
    let vec = Vec2::new(1., 2.);
    assert_approx_eq!(vec.x, 1.);
    assert_approx_eq!(vec.y, 2.);
}

#[modor::test]
fn create_with_z() {
    let vec = Vec2::new(1., 2.).with_z(3.);
    assert_approx_eq!(vec.x, 1.);
    assert_approx_eq!(vec.y, 2.);
    assert_approx_eq!(vec.z, 3.);
}

#[modor::test]
fn create_with_scale() {
    let vec = Vec2::new(1., 2.).with_scale(Vec2::new(5., 3.));
    assert_approx_eq!(vec.x, 5.);
    assert_approx_eq!(vec.y, 6.);
}

#[modor::test]
fn create_with_rotation() {
    let vec = Vec2::new(1., 2.).with_rotation(FRAC_PI_2);
    assert_approx_eq!(vec.x, -2.);
    assert_approx_eq!(vec.y, 1.);
}

#[modor::test]
fn create_with_magnitude() {
    let vec = Vec2::new(1., 2.).with_magnitude(20_f32.sqrt()).unwrap();
    assert_approx_eq!(vec.x, 2.);
    assert_approx_eq!(vec.y, 4.);
    assert!(Vec2::new(0., 0.).with_magnitude(2.).is_none());
}

#[modor::test]
fn calculate_rotation_between_2_vecs() {
    let rotation = Vec2::new(0.5, 0.5).rotation(Vec2::new(0.5, -0.5));
    assert_approx_eq!(rotation, -FRAC_PI_2);
    let rotation = Vec2::new(0.5, -0.5).rotation(Vec2::new(0.5, 0.5));
    assert_approx_eq!(rotation, FRAC_PI_2);
}

#[modor::test]
fn calculate_dot_product() {
    let dot = Vec2::new(1., 2.).dot(Vec2::new(3., 4.));
    assert_approx_eq!(dot, 11.);
}

#[modor::test]
fn calculate_mirror_vec() {
    let mirror = Vec2::new(0.7, 0.3).mirror(Vec2::new(2., 2.));
    assert_approx_eq!(mirror.x, 0.3);
    assert_approx_eq!(mirror.y, 0.7);
}

#[modor::test]
fn add_vec() {
    let new_vec = Vec2::new(1., 2.) + Vec2::new(3., 5.);
    assert_approx_eq!(new_vec.x, 4.);
    assert_approx_eq!(new_vec.y, 7.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec += Vec2::new(3., 5.);
    assert_approx_eq!(new_vec.x, 4.);
    assert_approx_eq!(new_vec.y, 7.);
}

#[modor::test]
fn sub_vec() {
    let new_vec = Vec2::new(1., 2.) - Vec2::new(3., 5.);
    assert_approx_eq!(new_vec.x, -2.);
    assert_approx_eq!(new_vec.y, -3.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec -= Vec2::new(3., 5.);
    assert_approx_eq!(new_vec.x, -2.);
    assert_approx_eq!(new_vec.y, -3.);
}

#[modor::test]
fn mul_float() {
    let new_vec = Vec2::new(1., 2.) * 5.;
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    let new_vec = 5. * Vec2::new(1., 2.);
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    let new_vec = 5. * Vec2::new(1., 2.);
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec *= 5.;
    assert_approx_eq!(new_vec.x, 5.);
    assert_approx_eq!(new_vec.y, 10.);
}

#[modor::test]
fn div_float() {
    let new_vec = Vec2::new(1., 2.) / 5.;
    assert_approx_eq!(new_vec.x, 0.2);
    assert_approx_eq!(new_vec.y, 0.4);
    let mut new_vec = Vec2::new(1., 2.);
    new_vec /= 5.;
    assert_approx_eq!(new_vec.x, 0.2);
    assert_approx_eq!(new_vec.y, 0.4);
}

#[modor::test]
fn neg_vec() {
    let new_vec = -Vec2::new(1., 2.);
    assert_approx_eq!(new_vec.x, -1.);
    assert_approx_eq!(new_vec.y, -2.);
}

#[modor::test]
fn sum_vecs() {
    let sum: Vec2 = [Vec2::new(1., 2.), Vec2::new(3., 4.), Vec2::new(5., 6.)]
        .into_iter()
        .sum();
    assert_approx_eq!(sum.x, 9.);
    assert_approx_eq!(sum.y, 12.);
    let sum: Vec2 = iter::empty().sum();
    assert_approx_eq!(sum.x, 0.);
    assert_approx_eq!(sum.y, 0.);
}

#[modor::test]
fn abs_diff_eq() {
    assert!(Vec2::new(1., 2.).abs_diff_eq(&Vec2::new(1., 2.), f32::EPSILON));
    assert!(Vec2::new(1., 2.).abs_diff_eq(&Vec2::new(1. + f32::EPSILON, 2.), f32::EPSILON));
    assert!(Vec2::new(1., 2.).abs_diff_eq(&Vec2::new(1., 2. + f32::EPSILON), f32::EPSILON));
    assert!(!Vec2::new(1., 2.).abs_diff_eq(&Vec2::new(1. + 2. * f32::EPSILON, 2.), f32::EPSILON));
    assert!(!Vec2::new(1., 2.).abs_diff_eq(&Vec2::new(1., 2. + 2. * f32::EPSILON), f32::EPSILON));
    assert_approx_eq!(Vec2::default_epsilon(), f32::EPSILON);
}

#[modor::test]
fn relative_eq() {
    assert!(Vec2::new(1., 2.).relative_eq(&Vec2::new(1., 2.), f32::EPSILON, 0.1));
    assert!(Vec2::new(1., 2.).relative_eq(&Vec2::new(0.91, 2.), f32::EPSILON, 0.1));
    assert!(Vec2::new(1., 2.).relative_eq(&Vec2::new(1., 1.81), f32::EPSILON, 0.1));
    assert!(!Vec2::new(1., 2.).relative_eq(&Vec2::new(0.9, 2.), f32::EPSILON, 0.1));
    assert!(!Vec2::new(1., 2.).relative_eq(&Vec2::new(1., 1.8), f32::EPSILON, 0.1));
    assert_approx_eq!(Vec2::default_max_relative(), f32::EPSILON);
}

#[modor::test]
fn ulps_eq() {
    assert!(Vec2::new(1., 2.).ulps_eq(&Vec2::new(1., 2.), f32::EPSILON, 1));
    assert!(Vec2::new(1., 2.).ulps_eq(&Vec2::new(1. + f32::EPSILON, 2.), f32::EPSILON, 1));
    assert!(Vec2::new(1., 2.).ulps_eq(&Vec2::new(1., 2. + 2. * f32::EPSILON), f32::EPSILON, 1));
    assert!(!Vec2::new(1., 2.).ulps_eq(&Vec2::new(1. + 2. * f32::EPSILON, 2.), f32::EPSILON, 1));
    assert!(!Vec2::new(1., 2.).ulps_eq(&Vec2::new(1., 2. + 3. * f32::EPSILON), f32::EPSILON, 1));
    assert_eq!(Vec2::default_max_ulps(), 4);
}

#[modor::test]
fn use_vector() {
    let vec = Vec2::new(1., 2.);
    assert_approx_eq!(vec.magnitude(), 5_f32.sqrt());
    assert_approx_eq!(vec.distance(Vec2::new(4., 3.)), 10_f32.sqrt());
}
