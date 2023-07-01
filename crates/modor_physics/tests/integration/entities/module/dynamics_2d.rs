use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};
use std::mem;
use std::time::Duration;

#[derive(Component, NoSystem)]
struct Source;

#[derive(Component, NoSystem)]
struct Destination;

#[modor_test]
fn update_velocity() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .component(Dynamics2D::new().with_velocity(Vec2::new(0.1, 0.2)));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.2, 2.4)))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.1, 0.2)))
        })
        .with_update::<(), _>(|d: &mut Dynamics2D| *d.velocity = Vec2::new(0.05, 0.1))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.3, 2.6)))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.05, 0.1)))
        })
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.4, 2.8)))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.05, 0.1)))
        });
}

#[modor_test]
fn update_angular_velocity() {
    let entity = EntityBuilder::new()
        .component(
            Transform2D::new()
                .with_position(Vec2::new(1., 2.))
                .with_rotation(PI),
        )
        .component(Dynamics2D::new().with_angular_velocity(FRAC_PI_4));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1., 2.)))
                .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -FRAC_PI_2))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_4))
        })
        .with_update::<(), _>(|d: &mut Dynamics2D| *d.angular_velocity = FRAC_PI_8)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -FRAC_PI_4))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_8))
        })
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 0.))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_8))
        });
}

#[modor_test]
fn remove_and_put_back_dynamics() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .component(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    let mut dynamics = Dynamics2D::new();
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .with_update::<(), _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .updated()
        .with_update::<(), _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(5., 2.)))
                .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(1., 0.)))
        });
}

#[modor_test]
fn move_dynamics() {
    let source = EntityBuilder::new()
        .component(Source)
        .component(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .component(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    let destination = EntityBuilder::new()
        .component(Destination)
        .component(Transform2D::new().with_position(Vec2::new(2., 1.)))
        .component(Dynamics2D::new());
    let mut dynamics = Dynamics2D::new();
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(source)
        .with_entity(destination)
        .updated()
        .with_update::<With<Source>, _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .with_update::<With<Destination>, _>(|d: &mut Dynamics2D| mem::swap(d, &mut dynamics))
        .updated()
        .assert::<With<Source>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 2.)))
        })
        .assert::<With<Destination>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(4., 1.)))
                .has(|d: &Dynamics2D| assert_approx_eq!(*d.velocity, Vec2::new(1., 0.)))
        });
}

#[modor_test]
fn create_with_relative_transform() {
    let entity = EntityBuilder::new()
        .component(Transform2D::new())
        .component(RelativeTransform2D::new())
        .component(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::from(Duration::from_secs(2)))
        .with_entity(entity)
        .updated()
        .assert::<With<Transform2D>>(1, |e| {
            e.has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(0., 0.)))
        });
}
