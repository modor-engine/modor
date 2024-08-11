use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, Body2DUpdater, Delta};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};
use std::time::Duration;

#[modor::test]
fn update_velocity() {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .velocity(Vec2::new(2., 1.))
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).position(&app), Vec2::new(4., 2.));
    app.update();
    assert_approx_eq!(body.get(&app).position(&app), Vec2::new(8., 4.));
}

#[modor::test]
fn update_angular_velocity() {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .angular_inertia(1.)
        .angular_velocity(FRAC_PI_4)
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), FRAC_PI_2);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), -PI);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), -FRAC_PI_2);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), 0.);
}

#[modor::test]
fn update_damping() {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .velocity(Vec2::new(2., 1.))
        .damping(0.5)
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).position(&app), Vec2::new(2., 1.) * 1.6);
}

#[modor::test]
fn update_angular_damping() {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .angular_velocity(FRAC_PI_4)
        .angular_damping(0.5)
        .angular_inertia(1.)
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), FRAC_PI_4 * 1.6);
}

#[modor::test(cases(
    equal_to_one = "1., Vec2::new(5., 2.5), Vec2::new(18., 9.)",
    equal_to_two = "2., Vec2::new(2.5, 1.25), Vec2::new(9., 4.5)"
))]
fn update_force_and_mass(mass: f32, expected_position1: Vec2, expected_position2: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .mass(mass)
        .force(Vec2::new(2., 1.))
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).position(&app), expected_position1);
    app.update();
    assert_approx_eq!(body.get(&app).position(&app), expected_position2);
}

#[modor::test(cases(
    equal_to_one = "1., 0.98174775, -2.7488935",
    equal_to_two = "2., 0.98174775 / 2., 1.767146"
))]
fn update_torque_and_angular_inertia(
    angular_inertia: f32,
    expected_rotation1: f32,
    expected_rotation2: f32,
) {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    Body2DUpdater::default()
        .angular_inertia(angular_inertia)
        .torque(FRAC_PI_8)
        .apply(&mut app, &body);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), expected_rotation1);
    app.update();
    assert_approx_eq!(body.get(&app).rotation(&app), expected_rotation2);
}

#[derive(FromApp)]
struct Root;

impl State for Root {
    fn init(&mut self, app: &mut App) {
        app.get_mut::<Delta>().duration = Duration::from_secs(2);
    }
}
