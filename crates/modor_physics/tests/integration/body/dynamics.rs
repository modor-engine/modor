use modor::log::Level;
use modor::{App, Node, RootNode, Visit};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, Delta};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};
use std::time::Duration;

#[modor::test]
fn update_velocity() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::ZERO);
    body(&mut app).velocity = Vec2::new(2., 1.);
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(4., 2.));
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(8., 4.));
}

#[modor::test]
fn update_angular_velocity() {
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, 0.);
    body(&mut app).angular_inertia = 1.;
    body(&mut app).angular_velocity = FRAC_PI_4;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, FRAC_PI_2);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, -PI);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, -FRAC_PI_2);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, 0.);
}

#[modor::test]
fn update_damping() {
    let mut app = App::new::<Root>(Level::Info);
    body(&mut app).velocity = Vec2::new(2., 1.);
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(4., 2.));
    body(&mut app).damping = 0.5;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(11.2, 5.6));
}

#[modor::test]
fn update_angular_damping() {
    let mut app = App::new::<Root>(Level::Info);
    body(&mut app).angular_inertia = 1.;
    body(&mut app).angular_velocity = FRAC_PI_4;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, FRAC_PI_2);
    body(&mut app).angular_damping = 0.5;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, -0.6 * PI);
}

#[modor::test(cases(
    equal_to_one = "1., Vec2::new(5., 2.5), Vec2::new(18., 9.)",
    equal_to_two = "2., Vec2::new(2.5, 1.25), Vec2::new(9., 4.5)"
))]
fn update_force_and_mass(mass: f32, expected_position1: Vec2, expected_position2: Vec2) {
    let mut app = App::new::<Root>(Level::Info);
    body(&mut app).mass = mass;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::ZERO);
    body(&mut app).force = Vec2::new(2., 1.);
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, expected_position1);
    app.update();
    assert_approx_eq!(body(&mut app).position, expected_position2);
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
    body(&mut app).angular_inertia = angular_inertia;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, 0.);
    body(&mut app).torque = FRAC_PI_8;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, expected_rotation1);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, expected_rotation2);
}

#[modor::test]
fn update_position() {
    let mut app = App::new::<Root>(Level::Info);
    body(&mut app).velocity = Vec2::new(2., 1.);
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(4., 2.));
    body(&mut app).position = Vec2::ZERO;
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::ZERO);
    app.update();
    assert_approx_eq!(body(&mut app).position, Vec2::new(4., 2.));
}

#[modor::test]
fn update_rotation() {
    let mut app = App::new::<Root>(Level::Info);
    body(&mut app).angular_inertia = 1.;
    body(&mut app).angular_velocity = FRAC_PI_4;
    app.update();
    app.update();
    assert_approx_eq!(body(&mut app).rotation, FRAC_PI_2);
    body(&mut app).rotation = 0.;
    app.update();
    assert_approx_eq!(body(&mut app).rotation, 0.);
    app.update();
    assert_approx_eq!(body(&mut app).rotation, FRAC_PI_2);
}

fn body(app: &mut App) -> &mut Body2D {
    &mut app.get_mut::<Root>().body
}

#[derive(Node, Visit)]
struct Root {
    body: Body2D,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        app.get_mut::<Delta>().duration = Duration::from_secs(2);
        Self {
            body: Body2D::new(app),
        }
    }
}
