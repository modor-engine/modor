use modor_math::Vec2;
use modor_physics::Transform2D;

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

#[modor_test]
#[allow(clippy::redundant_clone)]
fn clone() {
    let mut transform = Transform2D::new();
    transform.position = Vec2::new(1., 2.);
    transform.size = Vec2::new(3., 4.);
    transform.rotation = 5.;
    let cloned_transform = transform.clone();
    assert_approx_eq!(cloned_transform.position, Vec2::new(1., 2.));
    assert_approx_eq!(cloned_transform.size, Vec2::new(3., 4.));
    assert_approx_eq!(cloned_transform.rotation, 5.);
}
