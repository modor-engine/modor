use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    instance_2d, texture_target, Camera2D, Color, Material, MaterialType, Size, TEXTURE_TARGETS,
};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use std::f32::consts::FRAC_PI_2;

#[modor_test(disabled(macos, android, wasm))]
fn create_hidden() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(test_camera(Camera2D::hidden(CAMERA)))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .assert::<With<TestCamera>>(
            1,
            assert_position(Size::new(800, 600), Vec2::ZERO, Vec2::new(-2. / 3., 0.5)),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(
                Size::new(800, 600),
                Vec2::new(800., 600.),
                Vec2::new(2. / 3., -0.5),
            ),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(Size::new(600, 800), Vec2::ZERO, Vec2::new(-0.5, 2. / 3.)),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(
                Size::new(600, 800),
                Vec2::new(600., 800.),
                Vec2::new(0.5, -2. / 3.),
            ),
        );
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_one_target() {
    let missing_target_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(test_camera(Camera2D::new(CAMERA, TEXTURE_TARGETS.get(0))))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .with_update::<With<TestCamera>, _>(|c: &mut Camera2D| {
            c.target_keys[0] = TEXTURE_TARGETS.get(1);
        })
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#not_empty2"))
        .with_update::<With<TestCamera>, _>(|c: &mut Camera2D| {
            c.target_keys[0] = missing_target_key;
        })
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_many_targets() {
    let mut camera = Camera2D::new(CAMERA, TEXTURE_TARGETS.get(0));
    camera.target_keys.push(TEXTURE_TARGETS.get(1));
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(test_camera(camera))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#not_empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_transform() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(test_camera(Camera2D::new(CAMERA, TEXTURE_TARGETS.get(0))))
        .updated()
        .with_component::<With<TestCamera>, _>(|| {
            let mut transform = Transform2D::new();
            transform.position = Vec2::ONE * 0.5;
            transform.size = Vec2::ONE * 2.;
            transform.rotation = FRAC_PI_2;
            transform
        })
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty_offset1"))
        .assert::<With<TestCamera>>(
            1,
            assert_position(Size::new(800, 600), Vec2::ZERO, Vec2::new(1.5, 11. / 6.)),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(
                Size::new(800, 600),
                Vec2::new(800., 600.),
                Vec2::new(-0.5, -5. / 6.),
            ),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(Size::new(600, 800), Vec2::ZERO, Vec2::new(11. / 6., 1.5)),
        )
        .assert::<With<TestCamera>>(
            1,
            assert_position(
                Size::new(600, 800),
                Vec2::new(600., 800.),
                Vec2::new(-5. / 6., -0.5),
            ),
        )
        .with_deleted_components::<With<TestCamera>, Transform2D>()
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"));
}

#[modor_test(disabled(macos, android, wasm))]
fn recreate_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(test_camera(Camera2D::new(CAMERA, TEXTURE_TARGETS.get(0))))
        .updated()
        .with_deleted_entities::<With<TestCamera>>()
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .with_entity(test_camera(Camera2D::new(CAMERA, TEXTURE_TARGETS.get(0))))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"));
}

assertion_functions!(
    fn assert_position(
        camera: &Camera2D,
        surface_size: Size,
        surface_position: Vec2,
        world_position: Vec2,
    ) {
        assert_approx_eq!(
            camera.world_position(surface_size, surface_position),
            world_position
        );
    }
);

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(texture_target(0, Size::new(30, 20), true).component(Target1))
        .child_entity(texture_target(1, Size::new(20, 30), true).component(Target2))
        .child_entity(model())
}

fn model() -> impl BuiltEntity {
    instance_2d(CAMERA, MaterialType::Rectangle)
        .updated(|t: &mut Transform2D| t.position = Vec2::ONE * 0.25)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
        .updated(|m: &mut Material| m.color = Color::BLUE)
}

fn test_camera(camera: Camera2D) -> impl BuiltEntity {
    EntityBuilder::new().component(camera).component(TestCamera)
}

const CAMERA: ResKey<Camera2D> = ResKey::new("main");

#[derive(SingletonComponent, NoSystem)]
struct Target1;

#[derive(SingletonComponent, NoSystem)]
struct Target2;

#[derive(SingletonComponent, NoSystem)]
struct TestCamera;
