use modor_math::{Quat, Vec3};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[modor_test]
fn create_from_axis_angle() {
    let quat = Quat::default();
    assert_approx_eq!(quat.angle(), 0.);
    assert!(quat.axis().is_none());
    let quat = Quat::from_axis_angle(Vec3::new(1., 2., 3.), PI.mul_add(6., FRAC_PI_2));
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(axis.x, 1. / 14_f32.sqrt());
    assert_approx_eq!(axis.y, 2. / 14_f32.sqrt());
    assert_approx_eq!(axis.z, 3. / 14_f32.sqrt());
    let quat = Quat::from_axis_angle(Vec3::new(-1., -2., -3.), FRAC_PI_2 - 6. * PI);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, -1. / 14_f32.sqrt());
    assert_approx_eq!(axis.y, -2. / 14_f32.sqrt());
    assert_approx_eq!(axis.z, -3. / 14_f32.sqrt());
    let quat = Quat::from_axis_angle(Vec3::ZERO, FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, 0.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
}

#[modor_test]
fn create_from_x() {
    let quat = Quat::from_x(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, 1.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
}

#[modor_test]
fn create_from_y() {
    let quat = Quat::from_y(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, 0.);
    assert_approx_eq!(axis.y, 1.);
    assert_approx_eq!(axis.z, 0.);
}

#[modor_test]
fn create_from_z() {
    let quat = Quat::from_z(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, 0.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 1.);
}

#[modor_test]
fn create_with_scale() {
    let quat = Quat::from_x(FRAC_PI_4).with_scale(2.);
    let axis = quat.axis().unwrap();
    assert_approx_eq!(quat.angle(), FRAC_PI_2);
    assert_approx_eq!(axis.x, 1.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
}

#[modor_test]
fn create_with_rotation() {
    let quat = Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_approx_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_approx_eq!(axis.x, 1.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
    let quat = Quat::from_axis_angle(Vec3::new(0., 1., 0.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(0., 1., 0.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_approx_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_approx_eq!(axis.x, 0.);
    assert_approx_eq!(axis.y, 1.);
    assert_approx_eq!(axis.z, 0.);
    let quat = Quat::from_axis_angle(Vec3::new(0., 0., 1.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(0., 0., 1.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_approx_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_approx_eq!(axis.x, 0.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 1.);
}

#[modor_test]
fn multiply_quaternions() {
    let mut quat1 = Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_2);
    let quat2 = Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_4);
    let new_quat = quat1 * quat2;
    let axis = new_quat.axis().unwrap();
    assert_approx_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_approx_eq!(axis.x, 1.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
    quat1 *= quat2;
    let axis = quat1.axis().unwrap();
    assert_approx_eq!(quat1.angle(), 3. * FRAC_PI_4);
    assert_approx_eq!(axis.x, 1.);
    assert_approx_eq!(axis.y, 0.);
    assert_approx_eq!(axis.z, 0.);
}
