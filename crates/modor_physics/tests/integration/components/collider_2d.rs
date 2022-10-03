use modor::testing::TestApp;
use modor::{App, Built, EntityBuilder};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, DeltaTime, Dynamics2D, PhysicsModule, Transform2D,
};
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

struct TestEntity;

#[entity]
impl TestEntity {}

fn entity(
    transform: Transform2D,
    collider: Collider2D,
    with_dynamics: bool,
) -> impl Built<TestEntity> {
    EntityBuilder::new(TestEntity)
        .with(transform)
        .with_option(with_dynamics.then(Dynamics2D::new))
        .with(collider)
}

fn assert_collision(
    transform1: Transform2D,
    collider1: Collider2D,
    transform2: Transform2D,
    collider2: Collider2D,
    normal_1: Vec2,
    position1: Vec2,
    position2: Vec2,
) {
    assert_collision_internal(
        entity(transform1.clone(), collider1.clone(), false),
        entity(transform2.clone(), collider2.clone(), false),
        normal_1,
        position1,
        position2,
    );
    assert_collision_internal(
        entity(transform1, collider1, true),
        entity(transform2, collider2, true),
        normal_1,
        position1,
        position2,
    );
}

fn assert_collision_internal(
    entity1: impl Built<TestEntity>,
    entity2: impl Built<TestEntity>,
    normal_1: Vec2,
    position1: Vec2,
    position2: Vec2,
) {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .into();
    let entity1_id = app.create_entity(entity1);
    let entity2_id = app.create_entity(entity2);
    app.update();
    app.assert_entity(entity1_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity2_id);
        assert_approx_eq!(collision.normal, normal_1);
        assert_approx_eq!(collision.position, position1);
    });
    app.assert_entity(entity2_id).has(|c: &Collider2D| {
        assert_eq!(c.collisions().len(), 1);
        let collision = &c.collisions()[0];
        assert_eq!(collision.other_entity_id, entity1_id);
        assert_approx_eq!(collision.normal, -normal_1);
        assert_approx_eq!(collision.position, position2);
    });
}

fn assert_no_collision(
    transform1: Transform2D,
    collider1: Collider2D,
    transform2: Transform2D,
    collider2: Collider2D,
) {
    assert_no_collision_internal(
        entity(transform1.clone(), collider1.clone(), false),
        entity(transform2.clone(), collider2.clone(), false),
    );
    assert_no_collision_internal(
        entity(transform1, collider1, true),
        entity(transform2, collider2, true),
    );
}

fn assert_no_collision_internal(entity1: impl Built<TestEntity>, entity2: impl Built<TestEntity>) {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(Duration::from_secs(2)))
        .into();
    let entity1_id = app.create_entity(entity1);
    let entity2_id = app.create_entity(entity2);
    app.update();
    app.assert_entity(entity1_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
    app.assert_entity(entity2_id)
        .has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_collision_rectangle_rectangle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.353_553_4, 3.),
        Vec2::new(-1.530_330_1, 2.823_223_4),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-0.6, 3.4))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_collision_circle_circle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::circle(CollisionGroupIndex::Group0),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.292_893_2, 2.707_106_8),
        Vec2::new(-1.353_553_3, 2.646_446_7),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-0.6, 3.4))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::circle(CollisionGroupIndex::Group0),
    );
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn check_collision_circle_rectangle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.292_893_2, 2.707_106_8),
        Vec2::new(-1.353_553_3, 2.646_446_7),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroupIndex::Group0),
        Transform2D::new()
            .with_position(Vec2::new(-0.8, 3.2))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroupIndex::Group0),
    );
}
