use modor_math::Vec2;
use modor_physics::Transform2D;
use std::f32::consts::PI;

#[modor_test]
fn create_default_transform() {
    let body = Transform2D::default();
    assert_approx_eq!(*body.position, Vec2::ZERO);
    assert_approx_eq!(*body.size, Vec2::ONE);
    assert_approx_eq!(*body.rotation, 0.);
}

#[modor_test]
fn create_new_transform() {
    let body = Transform2D::new();
    assert_approx_eq!(*body.position, Vec2::ZERO);
    assert_approx_eq!(*body.size, Vec2::ONE);
    assert_approx_eq!(*body.rotation, 0.);
}
