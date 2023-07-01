use modor::{App, BuildableEntity, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, DeltaTime, Dynamics2D, PhysicsModule, Transform2D,
};
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollisionGroup;

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, _other: &Self) -> CollisionType {
        CollisionType::Sensor
    }
}

#[derive(Component, NoSystem)]
struct Entity1;

#[derive(Component, NoSystem)]
struct Entity2;

fn entity1(transform: Transform2D, collider: Collider2D, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Entity1)
        .component(transform)
        .component_option(with_dynamics.then(Dynamics2D::new))
        .component(collider)
}

fn entity2(transform: Transform2D, collider: Collider2D, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Entity2)
        .component(transform)
        .component_option(with_dynamics.then(Dynamics2D::new))
        .component(collider)
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
    for with_dynamics in [true, false] {
        assert_collision_internal(
            entity1(transform1.clone(), collider1.clone(), with_dynamics),
            entity2(transform2.clone(), collider2.clone(), with_dynamics),
            normal_1,
            position1,
            position2,
        );
    }
}

fn assert_collision_internal(
    entity1: impl BuildableEntity,
    entity2: impl BuildableEntity,
    normal_1: Vec2,
    position1: Vec2,
    position2: Vec2,
) {
    let mut entity2_id = 0;
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity1)
        .with_entity(entity2)
        .updated()
        .with_update::<With<Entity1>, _>(|c: &mut Collider2D| {
            entity2_id = c.collisions()[0].other_entity_id;
        })
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_approx_eq!(collision.normal, normal_1);
                assert_approx_eq!(collision.position, position1);
            })
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| {
                assert_eq!(c.collisions().len(), 1);
                let collision = &c.collisions()[0];
                assert_eq!(collision.other_entity_id, entity2_id - 2);
                assert_approx_eq!(collision.normal, -normal_1);
                assert_approx_eq!(collision.position, position2);
            })
        });
}

fn assert_no_collision(
    transform1: Transform2D,
    collider1: Collider2D,
    transform2: Transform2D,
    collider2: Collider2D,
) {
    for with_dynamics in [true, false] {
        assert_no_collision_internal(
            entity1(transform1.clone(), collider1.clone(), with_dynamics),
            entity2(transform2.clone(), collider2.clone(), with_dynamics),
        );
    }
}

fn assert_no_collision_internal(entity1: impl BuildableEntity, entity2: impl BuildableEntity) {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity1)
        .with_entity(entity2)
        .updated()
        .assert::<With<Entity1>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        })
        .assert::<With<Entity2>>(1, |e| {
            e.has(|c: &Collider2D| assert_eq!(c.collisions().len(), 0))
        });
}

#[modor_test]
fn check_collision_rectangle_rectangle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::rectangle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroup),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.353_553_4, 3.),
        Vec2::new(-1.530_330_1, 2.823_223_4),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::rectangle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-0.6, 3.4))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroup),
    );
}

#[modor_test]
fn check_collision_circle_circle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::circle(CollisionGroup),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.292_893_2, 2.707_106_8),
        Vec2::new(-1.353_553_3, 2.646_446_7),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-0.6, 3.4))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::circle(CollisionGroup),
    );
}

#[modor_test]
fn check_collision_circle_rectangle() {
    assert_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-1., 3.))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroup),
        Vec2::new(2_f32.sqrt() / 2., 2_f32.sqrt() / 2.),
        Vec2::new(-1.292_893_2, 2.707_106_8),
        Vec2::new(-1.353_553_3, 2.646_446_7),
    );
    assert_no_collision(
        Transform2D::new()
            .with_position(Vec2::new(-2., 2.))
            .with_size(Vec2::ONE * 2.),
        Collider2D::circle(CollisionGroup),
        Transform2D::new()
            .with_position(Vec2::new(-0.8, 3.2))
            .with_size(Vec2::ONE)
            .with_rotation(FRAC_PI_4),
        Collider2D::rectangle(CollisionGroup),
    );
}
