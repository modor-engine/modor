use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, DeltaTime, Dynamics2D, PhysicsModule, Transform2D,
};
use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CollisionGroup;

impl CollisionGroupRef for CollisionGroup {
    fn collision_type(&self, _other: &Self) -> CollisionType {
        CollisionType::Sensor
    }
}

#[modor_test]
fn update_position() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .component(Dynamics2D::new())
        .component(Collider2D::rectangle(CollisionGroup));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1., 2.)))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.position = Vec2::new(3., 4.))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 4.)))
        });
}

#[modor_test]
fn update_size() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new().with_size(Vec2::new(1., 2.)))
        .component(Dynamics2D::new())
        .component(Collider2D::rectangle(CollisionGroup));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(1., 2.)))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.size = Vec2::new(3., 4.))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(3., 4.)))
        });
}

#[modor_test]
fn update_rotation() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new().with_rotation(PI))
        .component(Dynamics2D::new())
        .component(Collider2D::rectangle(CollisionGroup));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -PI))
        })
        .with_update::<(), _>(|t: &mut Transform2D| *t.rotation = FRAC_PI_2)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_2))
        });
}
