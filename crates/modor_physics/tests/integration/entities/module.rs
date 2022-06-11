use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::{App, EntityBuilder};
use modor_physics::{
    Acceleration, DeltaTime, PhysicsModule, Position, RelativeAcceleration, RelativePosition,
    RelativeSize, RelativeVelocity, Shape, Size, Velocity,
};
use std::time::Duration;

struct TestEntity;

#[entity]
impl TestEntity {}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_absolute_velocity_and_position() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(Position::xyz(1., 2., 3.))
            .with(Velocity::xyz(4., 5., 6.))
            .with(Acceleration::xyz(7., 8., 9.)),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.x, 7.))
        .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.y, 8.))
        .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.z, 9.))
        .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.x, 7.0_f32.mul_add(delta_time, 4.)))
        .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.y, 8.0_f32.mul_add(delta_time, 5.)))
        .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.z, 9.0_f32.mul_add(delta_time, 6.)))
        .has::<Position, _>(|p| {
            assert_abs_diff_eq!(p.x, 7.0_f32.mul_add(delta_time, 4.).mul_add(delta_time, 1.));
        })
        .has::<Position, _>(|p| {
            assert_abs_diff_eq!(p.y, 8.0_f32.mul_add(delta_time, 5.).mul_add(delta_time, 2.));
        })
        .has::<Position, _>(|p| {
            assert_abs_diff_eq!(p.z, 9.0_f32.mul_add(delta_time, 6.).mul_add(delta_time, 3.));
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_relative_velocity_and_position() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(RelativePosition::xyz(1., 2., 3.))
            .with(RelativeVelocity::xyz(4., 5., 6.))
            .with(RelativeAcceleration::xyz(7., 8., 9.)),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has::<RelativeAcceleration, _>(|a| assert_abs_diff_eq!(a.x, 7.))
        .has::<RelativeAcceleration, _>(|a| assert_abs_diff_eq!(a.y, 8.))
        .has::<RelativeAcceleration, _>(|a| assert_abs_diff_eq!(a.z, 9.))
        .has::<RelativeVelocity, _>(|v| assert_abs_diff_eq!(v.x, 7.0_f32.mul_add(delta_time, 4.)))
        .has::<RelativeVelocity, _>(|v| assert_abs_diff_eq!(v.y, 8.0_f32.mul_add(delta_time, 5.)))
        .has::<RelativeVelocity, _>(|v| assert_abs_diff_eq!(v.z, 9.0_f32.mul_add(delta_time, 6.)))
        .has::<RelativePosition, _>(|p| {
            assert_abs_diff_eq!(p.x, 7.0_f32.mul_add(delta_time, 4.).mul_add(delta_time, 1.));
        })
        .has::<RelativePosition, _>(|p| {
            assert_abs_diff_eq!(p.y, 8.0_f32.mul_add(delta_time, 5.).mul_add(delta_time, 2.));
        })
        .has::<RelativePosition, _>(|p| {
            assert_abs_diff_eq!(p.z, 9.0_f32.mul_add(delta_time, 6.).mul_add(delta_time, 3.));
        });
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_absolute_position_from_relative_position() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity1_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(RelativePosition::xyz(1., 2., 3.))
            .with(Velocity::xyz(0.1, 0.2, 0.3))
            .with(Position::xyz(0., 0., 0.))
            .with(Size::xyz(1., 1., 1.)),
    );
    let entity2_id = app.create_child(
        entity1_id,
        EntityBuilder::new(TestEntity).with(Position::xyz(3., 2., 1.)),
    );
    let entity3_id = app.create_child(
        entity2_id,
        EntityBuilder::new(TestEntity)
            .with(RelativePosition::xyz(4., 5., 6.))
            .with(Position::xyz(0., 0., 0.))
            .with(Size::xyz(0.1, 0.2, 0.5)),
    );
    let entity4_id = app.create_child(
        entity3_id,
        EntityBuilder::new(TestEntity)
            .with(RelativePosition::xyz(7., 8., 9.))
            .with(Position::xyz(0., 0., 0.))
            .with(Size::xyz(1., 1., 1.)),
    );
    app.update();
    app.assert_entity(entity1_id)
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.x, 1.))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.y, 2.))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.z, 3.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.x, 1.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.y, 2.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.z, 3.));
    app.assert_entity(entity3_id)
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.x, 5.))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.y, 7.))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.z, 9.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.x, 4.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.y, 5.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.z, 6.));
    app.assert_entity(entity4_id)
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.x, 5.7))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.y, 8.6))
        .has::<Position, _>(|p| assert_abs_diff_eq!(p.z, 13.5))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.x, 7.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.y, 8.))
        .has::<RelativePosition, _>(|p| assert_abs_diff_eq!(p.z, 9.));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_absolute_size() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity1_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(Size::xyz(0., 0., 0.))
            .with(RelativeSize::xyz(1., 2., 3.)),
    );
    let entity2_id = app.create_child(entity1_id, EntityBuilder::new(TestEntity));
    let entity3_id = app.create_child(
        entity2_id,
        EntityBuilder::new(TestEntity)
            .with(Size::xyz(0., 0., 0.))
            .with(RelativeSize::xyz(0.1, 0.2, 0.5))
            .with(Shape::Rectangle2D),
    );
    let entity4_id = app.create_child(
        entity3_id,
        EntityBuilder::new(TestEntity)
            .with(Size::xyz(0., 0., 0.))
            .with(RelativeSize::xyz(0.5, 0.2, 0.1)),
    );
    app.update();
    app.assert_entity(entity1_id)
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.x, 1.))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.y, 2.))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.z, 3.))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.x, 1.))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.y, 2.))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.z, 3.));
    app.assert_entity(entity3_id)
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.x, 0.1))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.y, 0.4))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.z, 1.5))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.x, 0.1))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.y, 0.2))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.z, 0.5));
    app.assert_entity(entity4_id)
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.x, 0.05))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.y, 0.08))
        .has::<Size, _>(|s| assert_abs_diff_eq!(s.z, 0.15))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.x, 0.5))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.y, 0.2))
        .has::<RelativeSize, _>(|s| assert_abs_diff_eq!(s.z, 0.1));
}
