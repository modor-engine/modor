use crate::entities::module::{Action, MoveDst, TestEntity, Updates, DELTA_TIME};
use modor::testing::TestApp;
use modor::{App, EntityBuilder};
use modor_math::Vec2;
use modor_physics::{DeltaTime, Dynamics2D, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_velocity() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .with(Dynamics2D::new().with_velocity(Vec2::new(0.1, 0.2)));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.2, 2.4)))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.1, 0.2)));
    app.run_for_singleton(|u: &mut Updates| {
        u.actions.push(Action::SetVelocity(Vec2::new(0.05, 0.1)));
    });
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.4, 2.8)))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.05, 0.1)));
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1.5, 3.)))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(0.05, 0.1)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_angular_velocity() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(
            Transform2D::new()
                .with_position(Vec2::new(1., 2.))
                .with_rotation(PI),
        )
        .with(Dynamics2D::new().with_angular_velocity(FRAC_PI_4));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1., 2.)))
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -FRAC_PI_2))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_4));
    app.run_for_singleton(|u: &mut Updates| {
        u.actions.push(Action::SetAngularVelocity(FRAC_PI_8));
    });
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 0.))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_8));
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_4))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.angular_velocity, FRAC_PI_8));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn remove_and_put_back_dynamics() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .with(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    let entity_id = app.create_entity(entity);
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::RemoveDynamics));
    app.update();
    app.assert_entity(entity_id).has_not::<Dynamics2D>();
    app.update();
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::PutBackDynamics));
    app.update();
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(5., 2.)))
        .has(|b: &Dynamics2D| assert_approx_eq!(*b.velocity, Vec2::new(1., 0.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn create_with_relative_transform() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new())
        .with(RelativeTransform2D::new())
        .with(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(0., 0.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn move_dynamics() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let src = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .with(Dynamics2D::new().with_velocity(Vec2::new(1., 0.)));
    let dst = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_position(Vec2::new(2., 1.)))
        .with(MoveDst);
    let src_id = app.create_entity(src);
    let dst_id = app.create_entity(dst);
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::MoveDynamics));
    app.update();
    app.update();
    app.assert_entity(src_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 2.)));
    app.assert_entity(dst_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(4., 1.)))
        .has(|d: &Dynamics2D| assert_approx_eq!(*d.velocity, Vec2::new(1., 0.)));
}
