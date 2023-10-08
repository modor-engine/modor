use modor_math::Vec2;
use modor_physics_new::Transform2D;

#[modor_test]
fn create_default() {
    let transform = Transform2D::default();
    assert_approx_eq!(transform.position, Vec2::ZERO);
    assert_approx_eq!(transform.size, Vec2::ONE);
    assert_approx_eq!(transform.rotation, 0.);
}

#[modor_test]
fn create_new() {
    let transform = Transform2D::new();
    assert_approx_eq!(transform.position, Vec2::ZERO);
    assert_approx_eq!(transform.size, Vec2::ONE);
    assert_approx_eq!(transform.rotation, 0.);
}
