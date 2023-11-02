use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};
use std::time::Duration;

#[modor_test]
fn create_default() {
    let dynamics = Dynamics2D::default();
    assert_approx_eq!(dynamics.velocity, Vec2::ZERO);
    assert_approx_eq!(dynamics.angular_velocity, 0.);
}

#[modor_test]
fn create_new() {
    let dynamics = Dynamics2D::new();
    assert_approx_eq!(dynamics.velocity, Vec2::ZERO);
    assert_approx_eq!(dynamics.angular_velocity, 0.);
}

#[modor_test]
fn update_velocity() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::ZERO))
        .with_update::<(), _>(|d: &mut Dynamics2D| d.velocity = Vec2::new(2., 1.))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(4., 2.)))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(8., 4.)));
}

#[modor_test]
fn update_angular_velocity() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(0.))
        .with_update::<(), _>(|d: &mut Dynamics2D| d.angular_velocity = FRAC_PI_4)
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(FRAC_PI_2))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(-PI))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(-FRAC_PI_2))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(0.));
}

#[modor_test]
fn update_position() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .with_update::<(), _>(|d: &mut Dynamics2D| d.velocity = Vec2::new(2., 1.))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(4., 2.)))
        .with_update::<(), _>(|t: &mut Transform2D| t.position = Vec2::new(0., 0.))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(4., 2.)))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(8., 4.)));
}

#[modor_test]
fn update_rotation() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .assert::<With<Dynamics2D>>(1, assert_rotation(0.))
        .with_update::<(), _>(|d: &mut Dynamics2D| d.angular_velocity = FRAC_PI_4)
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(FRAC_PI_2))
        .with_update::<(), _>(|t: &mut Transform2D| t.rotation = 0.)
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(FRAC_PI_2))
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_rotation(-PI));
}

#[modor_test]
fn remove_dynamics() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .with_update::<(), _>(|d: &mut Dynamics2D| d.velocity = Vec2::new(2., 1.))
        .updated()
        .updated()
        .assert::<With<Transform2D>>(1, assert_position(Vec2::new(8., 4.)))
        .with_deleted_components::<(), Dynamics2D>()
        .updated()
        .assert::<With<Transform2D>>(1, assert_position(Vec2::new(8., 4.)))
        .with_component::<With<Transform2D>, _>(|| {
            let mut dynamics = Dynamics2D::new();
            dynamics.velocity = Vec2::new(1., 2.);
            dynamics
        })
        .updated()
        .assert::<With<Transform2D>>(1, assert_position(Vec2::new(10., 8.)));
}

#[modor_test]
fn remove_transform() {
    App::new()
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(2)))
        .with_entity(physics_object())
        .with_update::<(), _>(|d: &mut Dynamics2D| d.velocity = Vec2::new(2., 1.))
        .updated()
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(8., 4.)))
        .with_deleted_components::<(), Transform2D>()
        .updated()
        .with_component::<With<Dynamics2D>, _>(Transform2D::new)
        .updated()
        .assert::<With<Dynamics2D>>(1, assert_position(Vec2::new(4., 2.)));
}

assertion_functions!(
    fn assert_position(transform: &Transform2D, position: Vec2) {
        assert_approx_eq!(transform.position, position);
    }

    fn assert_rotation(transform: &Transform2D, rotation: f32) {
        assert_approx_eq!(transform.rotation, rotation);
    }
);

fn physics_object() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .component(Dynamics2D::new())
}
