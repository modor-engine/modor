use approx::assert_abs_diff_eq;
use modor_math::Quat;
use modor_physics::{AngularAcceleration, RelativeAngularAcceleration};
use std::f32::consts::{FRAC_PI_2, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_angular_acceleration() {
    let mut rotation = AngularAcceleration::from(Quat::from_z(FRAC_PI_2));
    let axis = rotation.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    *rotation = rotation.with_rotation(Quat::from_z(FRAC_PI_2));
    let quat: Quat = rotation.into();
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), PI, epsilon = 0.000_001);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn use_relative_angular_acceleration() {
    let mut rotation = RelativeAngularAcceleration::from(Quat::from_z(FRAC_PI_2));
    let axis = rotation.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), FRAC_PI_2);
    *rotation = rotation.with_rotation(Quat::from_z(FRAC_PI_2));
    let quat: Quat = rotation.into();
    let axis = quat.axis().unwrap();
    assert_abs_diff_eq!(axis.x, 0.);
    assert_abs_diff_eq!(axis.y, 0.);
    assert_abs_diff_eq!(axis.z, 1.);
    assert_abs_diff_eq!(rotation.angle(), PI, epsilon = 0.000_001);
}
