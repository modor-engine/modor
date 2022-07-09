use approx::assert_abs_diff_eq;
use modor_math::{Quat, Vec3};
use modor_physics::DynamicBody;
use std::f32::consts::{FRAC_PI_2, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_new_body() {
    let body = DynamicBody::new();
    assert_abs_diff_eq!(body.velocity.x, 0.);
    assert_abs_diff_eq!(body.velocity.y, 0.);
    assert_abs_diff_eq!(body.velocity.z, 0.);
    assert_abs_diff_eq!(body.acceleration.x, 0.);
    assert_abs_diff_eq!(body.acceleration.y, 0.);
    assert_abs_diff_eq!(body.acceleration.z, 0.);
    assert_abs_diff_eq!(body.angular_velocity.angle(), 0.);
    assert_abs_diff_eq!(body.angular_acceleration.angle(), 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_customized_body() {
    let body = DynamicBody::new()
        .with_velocity(Vec3::xyz(1., 2., 3.))
        .with_acceleration(Vec3::xyz(4., 5., 6.))
        .with_angular_velocity(Quat::from_z(PI))
        .with_angular_acceleration(Quat::from_z(FRAC_PI_2));
    assert_abs_diff_eq!(body.velocity.x, 1.);
    assert_abs_diff_eq!(body.velocity.y, 2.);
    assert_abs_diff_eq!(body.velocity.z, 3.);
    assert_abs_diff_eq!(body.acceleration.x, 4.);
    assert_abs_diff_eq!(body.acceleration.y, 5.);
    assert_abs_diff_eq!(body.acceleration.z, 6.);
    assert_abs_diff_eq!(body.angular_velocity.angle(), PI);
    assert_abs_diff_eq!(body.angular_acceleration.angle(), FRAC_PI_2);
}
