use approx::assert_abs_diff_eq;
use modor::testing::TestApp;
use modor::{App, EntityBuilder};
use modor_math::{Quat, Vec3};
use modor_physics::{DeltaTime, DynamicBody, PhysicsModule, RelativeTransform, Transform};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, FRAC_PI_8, PI};
use std::time::Duration;

struct TestEntity;

#[entity]
impl TestEntity {}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_absolute_acceleration() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(1., 2., 3.))
                    .with_size(Vec3::xyz(4., 5., 6.)),
            )
            .with(DynamicBody::new().with_acceleration(Vec3::xyz(0.1, 0.2, 0.3))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.x, 0.2))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.y, 0.4))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.z, 0.6))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, 1.4))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, 2.8))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 4.2));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_relative_acceleration_without_parent() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(5., 5., 5.))
                    .with_size(Vec3::xyz(5., 5., 5.)),
            )
            .with(RelativeTransform::new().with_position(Vec3::xyz(7., 8., 9.)))
            .with(DynamicBody::new().with_acceleration(Vec3::xyz(0.1, 0.2, 0.3))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.x, 0.2))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.y, 0.4))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.z, 0.6))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, 7.4))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, 8.8))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 10.2));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_relative_acceleration_with_parent() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let parent_id = app.create_entity(
        EntityBuilder::new(TestEntity).with(
            Transform::new()
                .with_position(Vec3::xyz(1., 2., 3.))
                .with_size(Vec3::xyz(4., 5., 6.))
                .with_rotation(Quat::from_z(PI)),
        ),
    );
    let entity_id = app.create_child(
        parent_id,
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(5., 5., 5.))
                    .with_size(Vec3::xyz(5., 5., 5.)),
            )
            .with(RelativeTransform::new().with_position(Vec3::xyz(7., 8., 9.)))
            .with(DynamicBody::new().with_acceleration(Vec3::xyz(0.1, 0.2, 0.3))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.x, 0.2))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.y, 0.4))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.velocity.z, 0.6))
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.position.unwrap().x, 7.4))
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.position.unwrap().y, 8.8))
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.position.unwrap().z, 10.2))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, -28.6, epsilon = 0.000_01))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, -42., epsilon = 0.000_01))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 64.2, epsilon = 0.000_01));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_absolute_angular_acceleration() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(1., 2., 3.))
                    .with_size(Vec3::xyz(4., 5., 6.)),
            )
            .with(DynamicBody::new().with_angular_acceleration(Quat::from_z(FRAC_PI_4))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.angle(), FRAC_PI_2))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().x, 0.))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().y, 0.))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().z, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.angle(), PI, epsilon = 0.000_01))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().x, 0.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().y, 0.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().z, 1.));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_relative_angular_acceleration_without_parent() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(5., 5., 5.))
                    .with_size(Vec3::xyz(5., 5., 5.)),
            )
            .with(RelativeTransform::new().with_rotation(Quat::from_z(FRAC_PI_2)))
            .with(DynamicBody::new().with_angular_acceleration(Quat::from_z(FRAC_PI_4))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.angle(), FRAC_PI_2))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().x, 0.))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().y, 0.))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().z, 1.))
        .has(|t: &Transform| {
            assert_abs_diff_eq!(t.rotation.angle(), 3. * FRAC_PI_2, epsilon = 0.000_01);
        })
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().x, 0.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().y, 0.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.axis().unwrap().z, 1.));
}

#[test]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
fn update_with_relative_angular_acceleration_with_parent() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let parent_id = app.create_entity(
        EntityBuilder::new(TestEntity).with(
            Transform::new()
                .with_position(Vec3::xyz(1., 2., 3.))
                .with_size(Vec3::xyz(4., 5., 6.))
                .with_rotation(Quat::from_z(PI)),
        ),
    );
    let entity_id = app.create_child(
        parent_id,
        EntityBuilder::new(TestEntity)
            .with(
                Transform::new()
                    .with_position(Vec3::xyz(5., 5., 5.))
                    .with_size(Vec3::xyz(5., 5., 5.)),
            )
            .with(RelativeTransform::new().with_rotation(Quat::from_z(FRAC_PI_2)))
            .with(DynamicBody::new().with_angular_acceleration(Quat::from_z(FRAC_PI_8))),
    );
    let delta_time = 2.;
    app.run_for_singleton(|t: &mut DeltaTime| t.set(Duration::from_secs_f32(delta_time)));
    app.update();
    app.assert_entity(entity_id)
        .has(|b: &DynamicBody| {
            assert_abs_diff_eq!(b.angular_velocity.angle(), FRAC_PI_4, epsilon = 0.000_01);
        })
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().x, 0.))
        .has(|b: &DynamicBody| assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().y, 0.))
        .has(|b: &DynamicBody| {
            assert_abs_diff_eq!(b.angular_velocity.axis().unwrap().z, 1., epsilon = 0.000_01);
        })
        .has(|t: &RelativeTransform| {
            assert_abs_diff_eq!(t.rotation.unwrap().angle(), PI, epsilon = 0.000_01);
        })
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.rotation.unwrap().axis().unwrap().x, 0.))
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.rotation.unwrap().axis().unwrap().y, 0.))
        .has(|t: &RelativeTransform| assert_abs_diff_eq!(t.rotation.unwrap().axis().unwrap().z, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.rotation.angle(), 2. * PI, epsilon = 0.000_01))
        .has(|t: &Transform| assert!(t.rotation.axis().is_none()));
}

#[test]
fn update_hierarchically() {
    let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
    let entity1_id = app.create_entity(
        EntityBuilder::new(TestEntity)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xyz(1., 2., 3.))
                    .with_size(Vec3::xyz(1., 1., 1.)),
            ),
    );
    let entity2_id = app.create_child(
        entity1_id,
        EntityBuilder::new(TestEntity)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xyz(0.1, 0.2, 0.3))
                    .with_size(Vec3::xyz(1., 1., 1.)),
            ),
    );
    let entity3_id = app.create_child(entity2_id, EntityBuilder::new(TestEntity));
    let entity4_id = app.create_child(
        entity3_id,
        EntityBuilder::new(TestEntity)
            .with(Transform::default())
            .with(
                RelativeTransform::new()
                    .with_position(Vec3::xyz(0., 0., 0.))
                    .with_size(Vec3::xyz(0.1, 0.2, 0.3)),
            ),
    );
    app.update();
    app.assert_entity(entity1_id)
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, 2.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 3.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.x, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.y, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.z, 1.));
    app.assert_entity(entity2_id)
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, 1.1))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, 2.2))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 3.3))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.x, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.y, 1.))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.z, 1.));
    app.assert_entity(entity4_id)
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.x, 1.1))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.y, 2.2))
        .has(|t: &Transform| assert_abs_diff_eq!(t.position.z, 3.3))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.x, 0.1))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.y, 0.2))
        .has(|t: &Transform| assert_abs_diff_eq!(t.size.z, 0.3));
}
