use modor_physics::RelativeTransform2D;

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
