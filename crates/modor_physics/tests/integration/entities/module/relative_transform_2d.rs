use crate::entities::module::{Action, TestEntity, Updates, DELTA_TIME};
use modor::testing::TestApp;
use modor::{App, EntityBuilder};
use modor_math::Vec2;
use modor_physics::{DeltaTime, PhysicsModule, RelativeTransform2D, Transform2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_relative_position() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let root = EntityBuilder::new(TestEntity)
        .with(
            Transform2D::new()
                .with_position(Vec2::new(1., 2.))
                .with_size(Vec2::new(2., 4.))
                .with_rotation(FRAC_PI_2),
        )
        .with(RelativeTransform2D::new().with_position(Vec2::new(3., 4.)));
    let relative_child = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_position(Vec2::new(0.1, 0.2)))
        .with(RelativeTransform2D::new().with_position(Vec2::new(0.5, 0.2)));
    let absolute_child = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_position(Vec2::new(0.3, 0.4)))
        .with(RelativeTransform2D::new());
    let root_id = app.create_entity(root);
    let relative_child_id = app.create_child(root_id, relative_child);
    let absolute_child_id = app.create_child(root_id, absolute_child);
    app.update();
    app.assert_entity(root_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(3., 4.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.position.unwrap(), Vec2::new(3., 4.)));
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(2.2, 5.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.position.unwrap(), Vec2::new(0.5, 0.2)));
    app.assert_entity(absolute_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(0.3, 0.4)));
    app.run_for_singleton(|u: &mut Updates| {
        u.actions
            .push(Action::SetRelativePosition(Some(Vec2::new(10., 20.))));
    });
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-77., 24.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.position.unwrap(), Vec2::new(10., 20.)));
    app.run_for_singleton(|u: &mut Updates| {
        u.actions.push(Action::SetPosition(Vec2::new(5., 10.)));
    });
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.position, Vec2::new(-77., 24.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.position.unwrap(), Vec2::new(10., 20.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_relative_size() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let root = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_size(Vec2::new(2., 4.)))
        .with(RelativeTransform2D::new().with_size(Vec2::new(3., 5.)));
    let relative_child = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_size(Vec2::new(2., 4.)))
        .with(RelativeTransform2D::new().with_size(Vec2::new(0.5, 0.2)));
    let absolute_child = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_size(Vec2::new(5., 10.)))
        .with(RelativeTransform2D::new());
    let root_id = app.create_entity(root);
    let relative_child_id = app.create_child(root_id, relative_child);
    let absolute_child_id = app.create_child(root_id, absolute_child);
    app.update();
    app.assert_entity(root_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(3., 5.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.size.unwrap(), Vec2::new(3., 5.)));
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(1.5, 1.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.size.unwrap(), Vec2::new(0.5, 0.2)));
    app.assert_entity(absolute_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(5., 10.)));
    app.run_for_singleton(|u: &mut Updates| {
        u.actions
            .push(Action::SetRelativeSize(Some(Vec2::new(10., 20.))));
    });
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(30., 100.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.size.unwrap(), Vec2::new(10., 20.)));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetSize(Vec2::new(6., 30.))));
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.size, Vec2::new(30., 100.)))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.size.unwrap(), Vec2::new(10., 20.)));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_relative_rotation() {
    let mut app: TestApp = App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(DeltaTime::build(DELTA_TIME))
        .into();
    let root = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_rotation(0.))
        .with(RelativeTransform2D::new().with_rotation(PI));
    let relative_child = EntityBuilder::new(TestEntity)
        .inherit_from(Updates::build())
        .with(Transform2D::new().with_rotation(0.))
        .with(RelativeTransform2D::new().with_rotation(FRAC_PI_2));
    let absolute_child = EntityBuilder::new(TestEntity)
        .with(Transform2D::new().with_rotation(FRAC_PI_4))
        .with(RelativeTransform2D::new());
    let root_id = app.create_entity(root);
    let relative_child_id = app.create_child(root_id, relative_child);
    let absolute_child_id = app.create_child(root_id, absolute_child);
    app.update();
    app.assert_entity(root_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, PI))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI));
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 3. * FRAC_PI_2))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), FRAC_PI_2));
    app.assert_entity(absolute_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, FRAC_PI_4));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetRelativeRotation(Some(PI))));
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 2. * PI))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI));
    app.run_for_singleton(|u: &mut Updates| u.actions.push(Action::SetRotation(FRAC_PI_2)));
    app.update();
    app.update();
    app.assert_entity(relative_child_id)
        .has(|t: &Transform2D| assert_approx_eq!(*t.rotation, 2. * PI))
        .has(|t: &RelativeTransform2D| assert_approx_eq!(t.rotation.unwrap(), PI));
}
