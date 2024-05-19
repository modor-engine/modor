use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, CollisionGroup, CollisionType, Delta, Impulse, Shape2D};
use std::time::Duration;

#[modor::test]
fn colliding_bodies_without_collision_group() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    app.update();
    assert!(body1(&mut app).collisions().is_empty());
    assert!(body2(&mut app).collisions().is_empty());
}

#[modor::test]
fn colliding_bodies_with_no_interaction() {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    app.update();
    app.update();
    assert!(body1(&mut app).collisions().is_empty());
    assert!(body2(&mut app).collisions().is_empty());
}

#[modor::test]
fn colliding_bodies_with_sensor() {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Sensor);
    app.update();
    app.update();
    let group1 = body1(&mut app).collision_group.clone().unwrap();
    let group2 = body2(&mut app).collision_group.clone().unwrap();
    let body = body1(&mut app);
    assert_approx_eq!(body.position, Vec2::ZERO);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.5);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * 0.75);
    assert_eq!(body.collisions()[0].other_index, 1);
    assert_eq!(body.collisions()[0].other_group_index, group2.index());
    assert!(body.is_colliding_with(&group2));
    assert!(!body.is_colliding_with(&group1));
    let body = body2(&mut app);
    assert_approx_eq!(body.position, Vec2::X);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * -0.25);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * -0.75);
    assert_eq!(body.collisions()[0].other_index, 0);
    assert_eq!(body.collisions()[0].other_group_index, group1.index());
    assert!(body.is_colliding_with(&group1));
    assert!(!body.is_colliding_with(&group2));
}

#[modor::test]
fn colliding_bodies_with_impulse() {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Impulse(Impulse::default()));
    body2(&mut app).mass = 1.;
    app.update();
    app.update();
    let group1 = body1(&mut app).collision_group.clone().unwrap();
    let group2 = body2(&mut app).collision_group.clone().unwrap();
    let body = body1(&mut app);
    assert_approx_eq!(body.position, Vec2::ZERO);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.5);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * 0.074_572);
    assert_eq!(body.collisions()[0].other_index, 1);
    assert_eq!(body.collisions()[0].other_group_index, group2.index());
    assert!(body.is_colliding_with(&group2));
    assert!(!body.is_colliding_with(&group1));
    let body = body2(&mut app);
    assert!(body.position.x > 1.1);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.425_427);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * -0.074_572);
    assert_eq!(body.collisions()[0].other_index, 0);
    assert_eq!(body.collisions()[0].other_group_index, group1.index());
    assert!(body.is_colliding_with(&group1));
    assert!(!body.is_colliding_with(&group2));
}

#[modor::test(cases(
    zero = "0., Vec2::new(0.25, 0.249_921)",
    one = "1., Vec2::new(0.228_143, 0.249_921)"
))]
fn set_friction(friction: f32, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Impulse(Impulse::new(0., friction)));
    configure_ground(&mut app);
    configure_rolling_ball(&mut app);
    app.update();
    app.update();
    assert_approx_eq!(body1(&mut app).position, Vec2::ZERO);
    assert_approx_eq!(body2(&mut app).position, expected_position);
}

#[modor::test(cases(
    zero = "0., Vec2::new(0., 0.222_098)",
    one = "1., Vec2::new(0., 0.341_609)"
))]
fn set_restitution(restitution: f32, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    app.root::<Delta>().duration = Duration::from_secs_f32(0.1);
    let impulse = Impulse::new(restitution, 0.5);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Impulse(impulse));
    configure_ground(&mut app);
    configure_falling_ball(&mut app);
    for _ in 0..11 {
        app.update();
    }
    assert_approx_eq!(body1(&mut app).position, Vec2::ZERO);
    assert_approx_eq!(body2(&mut app).position, expected_position);
}

#[modor::test(cases(
    less = "-1, Vec2::new(0., 0.341_609)",
    equal = "0, Vec2::new(0., 0.341_609)",
    greater = "1, Vec2::new(0., -0.0249_998)"
))]
fn set_dominance(dominance: i8, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    app.root::<Delta>().duration = Duration::from_secs_f32(0.1);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Impulse(Impulse::new(1., 0.5)));
    configure_ground(&mut app);
    configure_falling_ball(&mut app);
    body2(&mut app).dominance = dominance;
    for _ in 0..11 {
        app.update();
    }
    assert_approx_eq!(body1(&mut app).position, Vec2::ZERO);
    assert_approx_eq!(body2(&mut app).position, expected_position);
}

#[modor::test(cases(
    enabled = "true, Vec2::new(0., 0.255)",
    disabled = "false, Vec2::new(0., -4.)"
))]
fn set_ccd(is_enabled: bool, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Impulse(Impulse::new(1., 0.5)));
    configure_ground(&mut app);
    configure_falling_ball(&mut app);
    body2(&mut app).is_ccd_enabled = is_enabled;
    app.update();
    app.update();
    assert_approx_eq!(body1(&mut app).position, Vec2::ZERO);
    assert_approx_eq!(body2(&mut app).position, expected_position);
}

#[modor::test(cases(
    diagonal_rectangle = "Vec2::new(0.9, 0.9), Vec2::ONE, Shape2D::Rectangle, 1",
    horizontal_rectangle = "Vec2::X * 0.9, Vec2::ONE, Shape2D::Rectangle, 1",
    vectical_rectangle = "Vec2::Y * 0.9, Vec2::ONE, Shape2D::Rectangle, 1",
    diagonal_circle = "Vec2::new(0.9, 0.9), Vec2::ONE, Shape2D::Circle, 0",
    horizontal_circle = "Vec2::X * 0.9, Vec2::ONE, Shape2D::Circle, 1",
    vectical_circle = "Vec2::Y * 0.9, Vec2::ONE, Shape2D::Circle, 1",
    horizontal_circle_lower_height = "Vec2::X * 0.9, Vec2::new(1., 0.79), Shape2D::Circle, 0",
    vectical_circle_lower_height = "Vec2::Y * 0.9, Vec2::new(1., 0.79), Shape2D::Circle, 0",
))]
fn set_shape(position: Vec2, size: Vec2, shape: Shape2D, collision_count: usize) {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Sensor);
    body2(&mut app).position = position;
    body2(&mut app).size = size;
    body2(&mut app).shape = shape;
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), collision_count);
    assert_eq!(body2(&mut app).collisions().len(), collision_count);
}

#[modor::test]
fn update_size() {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Sensor);
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 1);
    assert_eq!(body2(&mut app).collisions().len(), 1);
    body2(&mut app).size.x = 0.5;
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 0);
    assert_eq!(body2(&mut app).collisions().len(), 0);
}

#[modor::test]
fn drop_body() {
    let mut app = App::new::<Root>(Level::Info);
    configure_colliding_groups(&mut app);
    configure_collision_type(&mut app, CollisionType::Sensor);
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 1);
    *body2(&mut app) = Body2D::new(&mut app.ctx());
    app.update();
    app.update();
    assert_eq!(body1(&mut app).collisions().len(), 0);
}

fn configure_colliding_groups(app: &mut App) {
    body1(app).collision_group = Some(group1(app).glob().clone());
    body2(app).collision_group = Some(group2(app).glob().clone());
}

fn configure_ground(app: &mut App) {
    body1(app).position = Vec2::ZERO;
    body1(app).size = Vec2::new(1., 0.01);
}

fn configure_falling_ball(app: &mut App) {
    body2(app).position = Vec2::Y * 1.;
    body2(app).size = Vec2::ONE * 0.5;
    body2(app).mass = 10.;
    body2(app).force = -20. * Vec2::Y;
    body2(app).shape = Shape2D::Circle;
}

fn configure_rolling_ball(app: &mut App) {
    body2(app).position = Vec2::Y * 0.251;
    body2(app).size = Vec2::ONE * 0.5;
    body2(app).mass = 10.;
    body2(app).force = Vec2::new(1., -0.1);
    body2(app).shape = Shape2D::Circle;
}

fn group1(app: &mut App) -> &CollisionGroup {
    &mut app.root::<Root>().group1
}

fn group2(app: &mut App) -> &CollisionGroup {
    &mut app.root::<Root>().group2
}

fn body1(app: &mut App) -> &mut Body2D {
    &mut app.root::<Root>().body1
}

fn body2(app: &mut App) -> &mut Body2D {
    &mut app.root::<Root>().body2
}

fn configure_collision_type(app: &mut App, collision_type: CollisionType) {
    app.root::<Root>().collision_type = Some(collision_type);
}

#[derive(Visit)]
struct Root {
    group1: CollisionGroup,
    group2: CollisionGroup,
    body1: Body2D,
    body2: Body2D,
    collision_type: Option<CollisionType>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.root::<Delta>().get_mut(ctx).duration = Duration::from_secs(2);
        let body1 = Body2D::new(ctx);
        let mut body2 = Body2D::new(ctx);
        body2.position = Vec2::X;
        body2.size = Vec2::new(2.5, 0.5);
        body2.update(ctx);
        Self {
            group1: CollisionGroup::new(ctx),
            group2: CollisionGroup::new(ctx),
            collision_type: None,
            body1,
            body2,
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        if let Some(collision_type) = self.collision_type {
            self.group1
                .add_interaction(ctx, self.group2.glob(), collision_type);
        }
    }
}
