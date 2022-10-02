use approx::assert_abs_diff_eq;
use modor_math::Vec2;
use modor_physics::Dynamics2D;
use std::f32::consts::PI;

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_default_body() {
    let body = Dynamics2D::default();
    assert_abs_diff_eq!(*body.velocity, Vec2::ZERO);
    assert_abs_diff_eq!(*body.angular_velocity, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_new_body() {
    let body = Dynamics2D::new();
    assert_abs_diff_eq!(*body.velocity, Vec2::ZERO);
    assert_abs_diff_eq!(*body.angular_velocity, 0.);
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_customized_body() {
    let body = Dynamics2D::new()
        .with_velocity(Vec2::new(1., 2.))
        .with_angular_velocity(PI);
    assert_abs_diff_eq!(*body.velocity, Vec2::new(1., 2.));
    assert_abs_diff_eq!(*body.angular_velocity, PI);
}
