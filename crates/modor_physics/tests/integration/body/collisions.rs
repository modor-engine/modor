use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{
    Body2D, Body2DUpdater, CollisionGroup, CollisionGroupUpdater, Delta, Impulse, Shape2D,
};
use std::time::Duration;

#[modor::test]
fn colliding_bodies_without_collision_group() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, false));
    app.update();
    assert!(res.body1.get(&app).collisions().is_empty());
    assert!(res.body2.get(&app).collisions().is_empty());
}

#[modor::test]
fn colliding_bodies_with_no_interaction() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    app.update();
    assert!(res.body1.get(&app).collisions().is_empty());
    assert!(res.body2.get(&app).collisions().is_empty());
}

#[modor::test]
fn colliding_bodies_with_sensor() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_sensor_interaction(&mut app);
    app.update();
    let body = res.body1.get(&app);
    assert_approx_eq!(body.position(&app), Vec2::ZERO);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.5);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * 0.75);
    assert_eq!(body.collisions()[0].other_index, 1);
    assert_eq!(body.collisions()[0].other_group_index, res.group2.index());
    assert!(body.is_colliding_with(&res.group2));
    assert!(!body.is_colliding_with(&res.group1));
    assert_eq!(body.collisions_with(&res.group2).count(), 1);
    assert_eq!(body.collisions_with(&res.group1).count(), 0);
    let body = res.body2.get(&app);
    assert_approx_eq!(body.position(&app), Vec2::X);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * -0.25);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * -0.75);
    assert_eq!(body.collisions()[0].other_index, 0);
    assert_eq!(body.collisions()[0].other_group_index, res.group1.index());
    assert!(body.is_colliding_with(&res.group1));
    assert!(!body.is_colliding_with(&res.group2));
    assert_eq!(body.collisions_with(&res.group1).count(), 1);
    assert_eq!(body.collisions_with(&res.group2).count(), 0);
}

#[modor::test]
fn colliding_bodies_with_impulse() {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_impulse_interaction(&mut app, Impulse::default());
    app.update();
    let body = res.body1.get(&app);
    assert_approx_eq!(body.position(&app), Vec2::ZERO);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.5);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * 0.001_063);
    assert_eq!(body.collisions()[0].other_index, 1);
    assert_eq!(body.collisions()[0].other_group_index, res.group2.index());
    assert!(body.is_colliding_with(&res.group2));
    assert!(!body.is_colliding_with(&res.group1));
    assert_eq!(body.collisions_with(&res.group2).count(), 1);
    assert_eq!(body.collisions_with(&res.group1).count(), 0);
    let body = res.body2.get(&app);
    assert!(body.position(&app).x > 1.1);
    assert_eq!(body.collisions().len(), 1);
    assert_approx_eq!(body.collisions()[0].position, Vec2::X * 0.498_936);
    assert_approx_eq!(body.collisions()[0].penetration, Vec2::X * -0.001_063);
    assert_eq!(body.collisions()[0].other_index, 0);
    assert_eq!(body.collisions()[0].other_group_index, res.group1.index());
    assert!(body.is_colliding_with(&res.group1));
    assert!(!body.is_colliding_with(&res.group2));
    assert_eq!(body.collisions_with(&res.group1).count(), 1);
    assert_eq!(body.collisions_with(&res.group2).count(), 0);
}

#[modor::test(cases(
    zero = "0., Vec2::new(0.25, 0.253_999)",
    one = "1., Vec2::new(0.222_000, 0.253_999)"
))]
fn set_friction(friction: f32, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_impulse_interaction(&mut app, Impulse::new(0., friction));
    res.configure_ground(&mut app);
    res.configure_rolling_ball(&mut app);
    app.update();
    assert_approx_eq!(res.body1.get(&app).position(&app), Vec2::ZERO);
    assert_approx_eq!(res.body2.get(&app).position(&app), expected_position);
}

#[modor::test(cases(
    zero = "0., Vec2::new(0., 0.234_789)",
    one = "1., Vec2::new(0., 0.374_636)"
))]
fn set_restitution(restitution: f32, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    app.get_mut::<Delta>().duration = Duration::from_secs_f32(0.1);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_impulse_interaction(&mut app, Impulse::new(restitution, 0.5));
    res.configure_ground(&mut app);
    res.configure_falling_ball(&mut app);
    for _ in 0..10 {
        app.update();
    }
    assert_approx_eq!(res.body1.get(&app).position(&app), Vec2::ZERO);
    assert_approx_eq!(res.body2.get(&app).position(&app), expected_position);
}

#[modor::test(cases(
    less = "-1, Vec2::new(0., 0.374_636)",
    equal = "0, Vec2::new(0., 0.374_636)",
    greater = "1, Vec2::new(0., -0.0249_998)"
))]
fn set_dominance(dominance: i8, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    app.get_mut::<Delta>().duration = Duration::from_secs_f32(0.1);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_impulse_interaction(&mut app, Impulse::new(1., 0.5));
    res.configure_ground(&mut app);
    res.configure_falling_ball(&mut app);
    Body2DUpdater::default()
        .dominance(dominance)
        .apply(&mut app, &res.body2);
    for _ in 0..10 {
        app.update();
    }
    assert_approx_eq!(res.body1.get(&app).position(&app), Vec2::ZERO);
    assert_approx_eq!(res.body2.get(&app).position(&app), expected_position);
}

#[modor::test(cases(
    enabled = "true, Vec2::new(0., 0.255)",
    disabled = "false, Vec2::new(0., -4.)"
))]
fn set_ccd(is_enabled: bool, expected_position: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_impulse_interaction(&mut app, Impulse::new(1., 0.5));
    res.configure_ground(&mut app);
    res.configure_falling_ball(&mut app);
    Body2DUpdater::default()
        .is_ccd_enabled(is_enabled)
        .apply(&mut app, &res.body2);
    app.update();
    assert_approx_eq!(res.body1.get(&app).position(&app), Vec2::ZERO);
    assert_approx_eq!(res.body2.get(&app).position(&app), expected_position);
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
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_sensor_interaction(&mut app);
    Body2DUpdater::default()
        .position(position)
        .size(size)
        .shape(shape)
        .apply(&mut app, &res.body2);
    app.update();
    assert_eq!(res.body1.get(&app).collisions().len(), collision_count);
    assert_eq!(res.body2.get(&app).collisions().len(), collision_count);
}

#[modor::test(cases(rectangle = "Shape2D::Rectangle", circle = "Shape2D::Circle"))]
fn update_size(shape: Shape2D) {
    let mut app = App::new::<Root>(Level::Info);
    let res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_sensor_interaction(&mut app);
    Body2DUpdater::default()
        .shape(shape)
        .apply(&mut app, &res.body2);
    app.update();
    assert_eq!(res.body1.get(&app).collisions().len(), 1);
    assert_eq!(res.body2.get(&app).collisions().len(), 1);
    Body2DUpdater::default()
        .for_size(|s| s.x = 0.5)
        .apply(&mut app, &res.body2);
    app.update();
    assert_eq!(res.body1.get(&app).collisions().len(), 0);
    assert_eq!(res.body2.get(&app).collisions().len(), 0);
}

#[modor::test]
fn drop_body() {
    let mut app = App::new::<Root>(Level::Info);
    let mut res = Resources::from_app_with(&mut app, |res, app| res.init(app, true));
    res.add_sensor_interaction(&mut app);
    app.update();
    assert_eq!(res.body1.get(&app).collisions().len(), 1);
    res.body2 = Glob::from_app(&mut app);
    app.update();
    assert_eq!(res.body1.get(&app).collisions().len(), 0);
}

#[derive(FromApp)]
struct Root;

impl State for Root {
    fn init(&mut self, app: &mut App) {
        app.get_mut::<Delta>().duration = Duration::from_secs(2);
    }
}

#[derive(FromApp)]
struct Resources {
    body1: Glob<Body2D>,
    body2: Glob<Body2D>,
    group1: Glob<CollisionGroup>,
    group2: Glob<CollisionGroup>,
}

impl Resources {
    fn init(&self, app: &mut App, assign_groups: bool) {
        Body2DUpdater::default()
            .collision_group(assign_groups.then(|| self.group1.to_ref()))
            .apply(app, &self.body1);
        Body2DUpdater::default()
            .position(Vec2::X)
            .size(Vec2::new(2.5, 3.))
            .collision_group(assign_groups.then(|| self.group2.to_ref()))
            .mass(1.)
            .apply(app, &self.body2);
    }

    fn add_sensor_interaction(&self, app: &mut App) {
        CollisionGroupUpdater::new(&self.group1).add_sensor(app, &self.group2);
    }

    fn add_impulse_interaction(&self, app: &mut App, impulse: Impulse) {
        CollisionGroupUpdater::new(&self.group1).add_impulse(app, &self.group2, impulse);
    }

    fn configure_ground(&self, app: &mut App) {
        Body2DUpdater::default()
            .position(Vec2::ZERO)
            .size(Vec2::new(1., 0.01))
            .apply(app, &self.body1);
    }

    fn configure_rolling_ball(&self, app: &mut App) {
        Body2DUpdater::default()
            .position(Vec2::Y * 0.251)
            .size(Vec2::ONE * 0.5)
            .mass(10.)
            .force(Vec2::new(1., -0.1))
            .shape(Shape2D::Circle)
            .apply(app, &self.body2);
    }

    fn configure_falling_ball(&self, app: &mut App) {
        Body2DUpdater::default()
            .position(Vec2::Y * 1.)
            .size(Vec2::ONE * 0.5)
            .mass(10.)
            .force(-20. * Vec2::Y)
            .shape(Shape2D::Circle)
            .apply(app, &self.body2);
    }
}
