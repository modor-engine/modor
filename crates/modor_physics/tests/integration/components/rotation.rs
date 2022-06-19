use approx::assert_abs_diff_eq;
use modor_math::{Quat, Vec3};
use modor_physics::{RelativeRotation, Rotation};
use std::f32::consts::{FRAC_PI_2, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_rotation() {
    let mut rotation = Rotation::from(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
    let axis = rotation.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    *rotation = rotation.with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
    let quat: Quat = rotation.into();
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), PI);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_relative_rotation() {
    let mut rotation = RelativeRotation::from(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
    let axis = rotation.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    *rotation = rotation.with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2));
    let quat: Quat = rotation.into();
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), PI);
}
