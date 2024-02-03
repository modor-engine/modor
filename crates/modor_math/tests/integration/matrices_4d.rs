use modor_internal::assert_approx_eq;
use modor_math::{Mat4, Quat, Vec2, Vec3};
use std::f32::consts::FRAC_PI_2;

#[modor::test]
fn create_from_array() {
    let mat = Mat4::from_array([
        [1., 2., 3., 4.],
        [5., 6., 7., 8.],
        [9., 10., 11., 12.],
        [13., 14., 15., 16.],
    ]);
    assert_approx_eq!(mat.to_array()[0][0], 1.);
    assert_approx_eq!(mat.to_array()[0][1], 2.);
    assert_approx_eq!(mat.to_array()[0][2], 3.);
    assert_approx_eq!(mat.to_array()[0][3], 4.);
    assert_approx_eq!(mat.to_array()[1][0], 5.);
    assert_approx_eq!(mat.to_array()[1][1], 6.);
    assert_approx_eq!(mat.to_array()[1][2], 7.);
    assert_approx_eq!(mat.to_array()[1][3], 8.);
    assert_approx_eq!(mat.to_array()[2][0], 9.);
    assert_approx_eq!(mat.to_array()[2][1], 10.);
    assert_approx_eq!(mat.to_array()[2][2], 11.);
    assert_approx_eq!(mat.to_array()[2][3], 12.);
    assert_approx_eq!(mat.to_array()[3][0], 13.);
    assert_approx_eq!(mat.to_array()[3][1], 14.);
    assert_approx_eq!(mat.to_array()[3][2], 15.);
    assert_approx_eq!(mat.to_array()[3][3], 16.);
}

#[modor::test]
fn create_from_position() {
    let mat = Mat4::from_position(Vec3::new(1., 2., 3.));
    assert_approx_eq!(mat.to_array()[0][0], 1.);
    assert_approx_eq!(mat.to_array()[0][1], 0.);
    assert_approx_eq!(mat.to_array()[0][2], 0.);
    assert_approx_eq!(mat.to_array()[0][3], 0.);
    assert_approx_eq!(mat.to_array()[1][0], 0.);
    assert_approx_eq!(mat.to_array()[1][1], 1.);
    assert_approx_eq!(mat.to_array()[1][2], 0.);
    assert_approx_eq!(mat.to_array()[1][3], 0.);
    assert_approx_eq!(mat.to_array()[2][0], 0.);
    assert_approx_eq!(mat.to_array()[2][1], 0.);
    assert_approx_eq!(mat.to_array()[2][2], 1.);
    assert_approx_eq!(mat.to_array()[2][3], 0.);
    assert_approx_eq!(mat.to_array()[3][0], 1.);
    assert_approx_eq!(mat.to_array()[3][1], 2.);
    assert_approx_eq!(mat.to_array()[3][2], 3.);
    assert_approx_eq!(mat.to_array()[3][3], 1.);
}

#[modor::test]
fn create_from_scale() {
    let mat = Mat4::from_scale(Vec3::new(4., 5., 6.));
    assert_approx_eq!(mat.to_array()[0][0], 4.);
    assert_approx_eq!(mat.to_array()[0][1], 0.);
    assert_approx_eq!(mat.to_array()[0][2], 0.);
    assert_approx_eq!(mat.to_array()[0][3], 0.);
    assert_approx_eq!(mat.to_array()[1][0], 0.);
    assert_approx_eq!(mat.to_array()[1][1], 5.);
    assert_approx_eq!(mat.to_array()[1][2], 0.);
    assert_approx_eq!(mat.to_array()[1][3], 0.);
    assert_approx_eq!(mat.to_array()[2][0], 0.);
    assert_approx_eq!(mat.to_array()[2][1], 0.);
    assert_approx_eq!(mat.to_array()[2][2], 6.);
    assert_approx_eq!(mat.to_array()[2][3], 0.);
    assert_approx_eq!(mat.to_array()[3][0], 0.);
    assert_approx_eq!(mat.to_array()[3][1], 0.);
    assert_approx_eq!(mat.to_array()[3][2], 0.);
    assert_approx_eq!(mat.to_array()[3][3], 1.);
}

#[modor::test]
fn mul_vec2() {
    let rotation = Quat::from_z(FRAC_PI_2).matrix();
    assert_approx_eq!(rotation * Vec2::new(1., 1.), Vec2::new(-1., 1.));
    assert_approx_eq!(Vec2::new(1., 1.) * rotation, Vec2::new(-1., 1.));
}

#[modor::test]
fn mul_vec3() {
    let rotation = Quat::from_x(FRAC_PI_2).matrix();
    assert_approx_eq!(rotation * Vec3::new(1., 1., 1.), Vec3::new(1., -1., 1.));
    assert_approx_eq!(Vec3::new(1., 1., 1.) * rotation, Vec3::new(1., -1., 1.));
}

#[modor::test]
fn mul_mat() {
    let translation = Mat4::from_position(Vec3::new(2., 4., 6.));
    let scale = Mat4::from_scale(Vec3::new(0.1, 0.2, 0.3));
    let rotation = Quat::from_x(FRAC_PI_2).matrix();
    assert_approx_eq!(
        rotation * scale * translation * Vec3::new(1., 1., 1.),
        Vec3::new(2.1, 3.8, 6.3)
    );
    let rotation = Quat::from_y(FRAC_PI_2).matrix();
    assert_approx_eq!(
        rotation * scale * translation * Vec3::new(1., 1., 1.),
        Vec3::new(2.1, 4.2, 5.7)
    );
    let rotation = Quat::from_z(FRAC_PI_2).matrix();
    assert_approx_eq!(
        rotation * scale * translation * Vec3::new(1., 1., 1.),
        Vec3::new(1.9, 4.2, 6.3)
    );
}
