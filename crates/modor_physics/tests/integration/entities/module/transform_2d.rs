use crate::entities::module::{Action, TestEntity, Updates, DELTA_TIME};
use modor::testing::TestApp;
use modor::{App, EntityBuilder};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupIndex, DeltaTime, Dynamics2D, PhysicsModule, Transform2D,
};
use std::f32::consts::{FRAC_PI_2, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_position() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_position(Vec2::new(1., 2.)))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(1., 2.)));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetPosition(Vec2::new(3., 4.))));
    app.update();
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 4.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_size() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_size(Vec2::new(1., 2.)))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(1., 2.)));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetSize(Vec2::new(4., 3.))));
    app.update();
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(4., 3.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_rotation() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let entity = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_rotation(PI))
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupIndex::Group0));
    let entity_id = app.create_entity(entity);
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, -PI));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetRotation(FRAC_PI_2)));
    app.update();
    app.update();
    app.assert_entity(entity_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_2));
}
