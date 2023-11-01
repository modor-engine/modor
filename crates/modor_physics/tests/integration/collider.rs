use modor::{App, BuiltEntity, Entity, EntityBuilder, Query, With};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroup, CollisionType, DeltaTime, Dynamics2D, Transform2D,
};
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

#[modor_test]
fn create_rectangle() {
    let collider = Collider2D::rectangle(GROUP1);
    assert!(collider.collisions().is_empty());
}

#[modor_test]
fn create_circle() {
    let collider = Collider2D::circle(GROUP1);
    assert!(collider.collisions().is_empty());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn check_collisions_for_collided_shapes(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_entity(circle(GROUP2, with_dynamics))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn update_position(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_entity(circle(GROUP2, with_dynamics))
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = Vec2::new(5., 5.))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Collider2D>>(2, assert_no_collision())
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = circle_position())
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle())
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = Vec2::new(5., 5.))
        .updated()
        .assert::<With<Collider2D>>(2, assert_no_collision());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn update_size(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_entity(circle(GROUP2, with_dynamics))
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.size = Vec2::ONE * 0.01)
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Collider2D>>(2, assert_no_collision())
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| {
            t.size = Vec2::ONE * 2. * circle_radius();
        })
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = Vec2::ZERO)
        .updated()
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = circle_position())
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn update_rotation(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_update::<With<Rectangle>, _>(|t: &mut Transform2D| t.rotation = 0.)
        .with_entity(circle(GROUP2, with_dynamics))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Rectangle>>(1, assert_different_rectangle())
        .with_update::<With<Rectangle>, _>(|t: &mut Transform2D| t.rotation = FRAC_PI_4)
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test]
fn update_velocity() {
    App::new()
        .with_entity(rectangle(GROUP1, false))
        .with_entity(circle(GROUP2, true))
        .with_update::<With<Circle>, _>(|t: &mut Transform2D| t.position = Vec2::new(5., 5.))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Collider2D>>(2, assert_no_collision())
        .with_update::<With<Circle>, _>(|t: &mut Dynamics2D| {
            t.velocity = Vec2::new(-5., -5.) + circle_position();
        })
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test]
fn update_angular_velocity() {
    App::new()
        .with_entity(rectangle(GROUP1, true))
        .with_update::<With<Rectangle>, _>(|t: &mut Transform2D| t.rotation = 0.)
        .with_entity(circle(GROUP2, false))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .assert::<With<Rectangle>>(1, assert_different_rectangle())
        .with_update::<With<Rectangle>, _>(|t: &mut Dynamics2D| t.angular_velocity = FRAC_PI_4)
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn remove_transform(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_entity(circle(GROUP2, with_dynamics))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .with_deleted_components::<With<Collider2D>, Transform2D>()
        .updated()
        .with_component::<With<Rectangle>, _>(|| {
            let mut transform = Transform2D::new();
            transform.size = Vec2::new(2., 1.);
            transform.rotation = FRAC_PI_4;
            transform
        })
        .with_component::<With<Circle>, _>(|| {
            let mut transform = Transform2D::new();
            transform.position = circle_position();
            transform.size = Vec2::new(circle_radius() * 2., 10.);
            transform
        })
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test]
fn remove_dynamics() {
    App::new()
        .with_entity(rectangle(GROUP1, true))
        .with_entity(circle(GROUP2, true))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .with_deleted_components::<With<Collider2D>, Dynamics2D>()
        .updated()
        .with_component::<With<Transform2D>, _>(Dynamics2D::new)
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

#[modor_test(cases(with_dynamics = "true", without_dynamics = "false"))]
fn remove_collider(with_dynamics: bool) {
    App::new()
        .with_entity(rectangle(GROUP1, with_dynamics))
        .with_entity(circle(GROUP2, with_dynamics))
        .with_entity(modor_physics::module())
        .with_update::<(), _>(|d: &mut DeltaTime| d.set(Duration::from_secs(1)))
        .with_entity(group1())
        .with_entity(group2())
        .updated()
        .with_deleted_components::<With<Transform2D>, Collider2D>()
        .updated()
        .with_component::<With<Rectangle>, _>(|| Collider2D::rectangle(GROUP1))
        .with_component::<With<Circle>, _>(|| Collider2D::circle(GROUP2))
        .updated()
        .assert::<With<Rectangle>>(1, assert_rectangle())
        .assert::<With<Circle>>(1, assert_circle());
}

assertion_functions!(
    fn assert_no_collision(collider: &Collider2D) {
        assert_eq!(collider.collisions().len(), 0);
    }

    fn assert_rotation(transform: &Transform2D, rotation: f32) {
        assert_approx_eq!(transform.rotation, rotation);
    }

    fn assert_rectangle(collider: &Collider2D) {
        let collision_position = rectangle_collision_position();
        let other_collision_position = circle_collision_position();
        let penetration = collision_position - other_collision_position;
        assert_eq!(collider.collisions().len(), 1);
        assert_eq!(collider.collisions()[0].other_entity_id, CIRCLE_ID);
        assert_eq!(collider.collisions()[0].other_group_key, GROUP2);
        assert_approx_eq!(collider.collisions()[0].position, collision_position);
        assert_approx_eq!(collider.collisions()[0].penetration, penetration);
    }

    fn assert_different_rectangle(collider: &Collider2D) {
        let collision_position = rectangle_collision_position();
        assert_eq!(collider.collisions().len(), 1);
        assert_eq!(collider.collisions()[0].other_entity_id, CIRCLE_ID);
        assert_eq!(collider.collisions()[0].other_group_key, GROUP2);
        let position = collider.collisions()[0].position;
        assert_approx_eq!(position, Vec2::new(collision_position.x * 2., 0.5));
        assert_approx_eq!(collider.collisions()[0].penetration, Vec2::new(0., 0.5));
    }

    fn assert_circle(collider: &Collider2D) {
        let collision_position = circle_collision_position();
        let other_collision_position = rectangle_collision_position();
        let penetration = collision_position - other_collision_position;
        assert_eq!(collider.collisions().len(), 1);
        assert_eq!(collider.collisions()[0].other_entity_id, RECTANGLE_ID);
        assert_eq!(collider.collisions()[0].other_group_key, GROUP1);
        assert_approx_eq!(collider.collisions()[0].position, collision_position);
        assert_approx_eq!(collider.collisions()[0].penetration, penetration);
    }
);

fn rectangle(group: ResKey<CollisionGroup>, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Rectangle)
        .component(Transform2D::new())
        .with(|t| t.size = Vec2::new(2., 1.))
        .with(|t| t.rotation = FRAC_PI_4)
        .component(Collider2D::rectangle(group))
        .component_option(with_dynamics.then(Dynamics2D::new))
}

fn circle(group: ResKey<CollisionGroup>, with_dynamics: bool) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Circle)
        .component(Transform2D::new())
        .with(|t| t.position = circle_position())
        .with(|t| t.size = Vec2::new(circle_radius() * 2., 10.))
        .component(Collider2D::circle(group))
        .component_option(with_dynamics.then(Dynamics2D::new))
}

fn rectangle_collision_position() -> Vec2 {
    circle_position() / 2.
}

fn circle_collision_position() -> Vec2 {
    circle_position() - (circle_position() - Vec2::Y * circle_radius()).with_rotation(-FRAC_PI_4)
}

fn group1() -> CollisionGroup {
    CollisionGroup::new(GROUP1, |k| {
        if k == GROUP1 {
            CollisionType::Impulse
        } else if k == GROUP2 {
            CollisionType::Sensor
        } else {
            CollisionType::None
        }
    })
}

fn group2() -> CollisionGroup {
    CollisionGroup::new(GROUP2, |_| CollisionType::None)
}

fn circle_radius() -> f32 {
    0.5_f32.sqrt()
}

fn circle_position() -> Vec2 {
    Vec2::new(-circle_radius(), circle_radius())
}

const RECTANGLE_ID: usize = 0;
const CIRCLE_ID: usize = 1;
const GROUP1: ResKey<CollisionGroup> = ResKey::new("group1");
const GROUP2: ResKey<CollisionGroup> = ResKey::new("group2");

#[derive(SingletonComponent, NoSystem)]
struct Rectangle;

#[derive(SingletonComponent)]
struct Circle;

#[systems]
impl Circle {
    #[run_after(component(Collider2D))]
    fn check_collisions(collider: &Collider2D, entities: Query<'_, Entity<'_>>) {
        if collider.collisions().is_empty() {
            assert_eq!(collider.collided(&entities).count(), 0);
            assert_eq!(collider.collided_as(&entities, GROUP1).count(), 0);
        } else {
            let collisions: Vec<_> = collider.collided(&entities).collect();
            assert_eq!(collisions.len(), 1);
            assert_eq!(collisions[0].0.other_entity_id, RECTANGLE_ID);
            assert_eq!(collisions[0].1.id(), RECTANGLE_ID);
            let collisions: Vec<_> = collider.collided_as(&entities, GROUP1).collect();
            assert_eq!(collisions.len(), 1);
            assert_eq!(collisions[0].0.other_entity_id, RECTANGLE_ID);
            assert_eq!(collisions[0].1.id(), RECTANGLE_ID);
        }
        assert_eq!(collider.collided_as(&entities, GROUP2).count(), 0);
    }
}
