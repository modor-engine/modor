use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::{Body2D, CollisionGroup, Shape2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

#[modor::test]
fn create_default() {
    let mut app = App::new::<Root>(Level::Info);
    let body = Glob::<Body2D>::from_app(&mut app);
    app.update();
    let body_ref = body.get(&app);
    assert_approx_eq!(body_ref.position(&app), Vec2::ZERO);
    assert_approx_eq!(body_ref.size(), Vec2::ONE);
    assert_approx_eq!(body_ref.rotation(&app), 0.);
    assert_approx_eq!(body_ref.velocity(&app), Vec2::ZERO);
    assert_approx_eq!(body_ref.angular_velocity(&app), 0.);
    assert_approx_eq!(body_ref.force(&app), Vec2::ZERO);
    assert_approx_eq!(body_ref.torque(&app), 0.);
    assert_approx_eq!(body_ref.mass(), 0.);
    assert_approx_eq!(body_ref.angular_inertia(), 0.);
    assert_approx_eq!(body_ref.damping(&app), 0.);
    assert_approx_eq!(body_ref.angular_damping(&app), 0.);
    assert_eq!(body_ref.dominance(&app), 0);
    assert!(!body_ref.is_ccd_enabled(&app));
    assert!(body_ref.collision_group().is_none());
    assert_eq!(body_ref.shape(&app), Shape2D::Rectangle);
}

#[modor::test]
fn update_fields() {
    let mut app = App::new::<Root>(Level::Info);
    let group = Glob::<CollisionGroup>::from_app(&mut app);
    let body = Glob::<Body2D>::from_app(&mut app);
    body.updater()
        .position(Vec2::new(1., 2.))
        .size(Vec2::new(1.1, 2.1))
        .rotation(FRAC_PI_2)
        .velocity(Vec2::new(3., 4.))
        .angular_velocity(FRAC_PI_4)
        .force(Vec2::new(5., 6.))
        .torque(FRAC_PI_8)
        .mass(0.1)
        .angular_inertia(PI)
        .damping(0.2)
        .angular_damping(0.3)
        .dominance(10)
        .is_ccd_enabled(true)
        .collision_group(group.to_ref())
        .shape(Shape2D::Circle)
        .apply(&mut app);
    let body_ref = body.get(&app);
    assert_approx_eq!(body_ref.position(&app), Vec2::new(1., 2.));
    assert_approx_eq!(body_ref.size(), Vec2::new(1.1, 2.1));
    assert_approx_eq!(body_ref.rotation(&app), FRAC_PI_2);
    assert_approx_eq!(body_ref.velocity(&app), Vec2::new(3., 4.));
    assert_approx_eq!(body_ref.angular_velocity(&app), FRAC_PI_4);
    assert_approx_eq!(body_ref.force(&app), Vec2::new(5., 6.));
    assert_approx_eq!(body_ref.torque(&app), FRAC_PI_8);
    assert_approx_eq!(body_ref.damping(&app), 0.2);
    assert_approx_eq!(body_ref.angular_damping(&app), 0.3);
    assert_approx_eq!(body_ref.mass(), 0.1);
    assert_approx_eq!(body_ref.angular_inertia(), PI);
    assert_eq!(body_ref.dominance(&app), 10);
    assert!(body_ref.is_ccd_enabled(&app));
    assert_eq!(body_ref.collision_group(), &Some(group.to_ref()));
    assert_eq!(body_ref.shape(&app), Shape2D::Circle);
}

#[derive(FromApp, State)]
struct Root;
