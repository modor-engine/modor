use approx::assert_abs_diff_eq;
use modor_math::{Quat, Vec3};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_axis_angle() {
    let quat = Quat::default();
    assert_abs_diff_eq!(quat.angle(), 0.);
    assert!(quat.axis().is_none());
    let quat = Quat::from_axis_angle(Vec3::new(1., 2., 3.), PI.mul_add(6., FRAC_PI_2));
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2, epsilon = 0.000_001);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 1. / 14_f32.sqrt());
    assert_abs_diff_eq!(axis.y, 2. / 14_f32.sqrt());
    assert_abs_diff_eq!(axis.z, 3. / 14_f32.sqrt());
    let quat = Quat::from_axis_angle(Vec3::new(-1., -2., -3.), FRAC_PI_2 - 6. * PI);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2, epsilon = 0.00001);
    assert_abs_diff_eq!(axis.x, -1. / 14_f32.sqrt());
    assert_abs_diff_eq!(axis.y, -2. / 14_f32.sqrt());
    assert_abs_diff_eq!(axis.z, -3. / 14_f32.sqrt());
    let quat = Quat::from_axis_angle(Vec3::ZERO, FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2, epsilon = 0.00001);
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_x() {
    let quat = Quat::from_x(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2);
    assert_abs_diff_eq!(axis.x, 1.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_y() {
    let quat = Quat::from_y(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2);
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 1.);
    assert_abs_diff_eq!(axis.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_from_z() {
    let quat = Quat::from_z(FRAC_PI_2);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2);
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_scale() {
    let quat = Quat::from_x(FRAC_PI_4).with_scale(2.);
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(quat.angle(), FRAC_PI_2, epsilon = 0.000_001);
    assert_abs_diff_eq!(axis.x, 1.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_rotation() {
    let quat = Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(1., 0., 0.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_abs_diff_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_abs_diff_eq!(axis.x, 1.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 0.);
    let quat = Quat::from_axis_angle(Vec3::new(0., 1., 0.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(0., 1., 0.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_abs_diff_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 1.);
    assert_abs_diff_eq!(axis.z, 0.);
    let quat = Quat::from_axis_angle(Vec3::new(0., 0., 1.), FRAC_PI_2);
    let new_quat = quat.with_rotation(Quat::from_axis_angle(Vec3::new(0., 0., 1.), FRAC_PI_4));
    let axis = new_quat.axis().unwrap();
    assert_abs_diff_eq!(new_quat.angle(), 3. * FRAC_PI_4);
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
}
