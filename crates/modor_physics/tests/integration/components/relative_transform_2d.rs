use modor_math::Vec2;
use modor_physics::RelativeTransform2D;
use std::f32::consts::PI;

#[modor_test]
fn create_default_transform() {
    let body = RelativeTransform2D::default();
    assert!(body.position.is_none());
    assert!(body.size.is_none());
    assert!(body.rotation.is_none());
}

#[modor_test]
fn create_new_transform() {
    let body = RelativeTransform2D::new();
    assert!(body.position.is_none());
    assert!(body.size.is_none());
    assert!(body.rotation.is_none());
}

#[modor_test]
fn create_customized_transform() {
    let body = RelativeTransform2D::new()
        .with_position(Vec2::new(1., 2.))
        .with_size(Vec2::new(3., 4.))
        .with_rotation(PI);
    assert_approx_eq!(body.position.unwrap(), Vec2::new(1., 2.));
    assert_approx_eq!(body.size.unwrap(), Vec2::new(3., 4.));
    assert_approx_eq!(body.rotation.unwrap(), PI);
}
