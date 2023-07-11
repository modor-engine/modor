use modor_math::Vec2;
use modor_physics::Dynamics2D;

#[modor_test]
fn create_default_body() {
    let body = Dynamics2D::default();
    assert_approx_eq!(*body.velocity, Vec2::ZERO);
    assert_approx_eq!(*body.angular_velocity, 0.);
}

#[modor_test]
fn create_new_body() {
    let body = Dynamics2D::new();
    assert_approx_eq!(*body.velocity, Vec2::ZERO);
    assert_approx_eq!(*body.angular_velocity, 0.);
}
