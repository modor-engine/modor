use modor::{App, BuiltEntity, EntityAssertions, EntityBuilder, EntityFilter, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
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
        .with_entity(Camera2D::hidden(CAMERA))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .assert::<With<Camera2D>>(
            1,
            assert_position(Size::new(800, 600), Vec2::ZERO, Vec2::new(-2. / 3., 0.5)),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(
                Size::new(800, 600),
                Vec2::new(800., 600.),
                Vec2::new(2. / 3., -0.5),
            ),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(Size::new(600, 800), Vec2::ZERO, Vec2::new(-0.5, 2. / 3.)),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(
                Size::new(600, 800),
                Vec2::new(600., 800.),
                Vec2::new(0.5, -2. / 3.),
            ),
        );
}

fn assert_position<F>(
    surface_size: Size,
    surface_position: Vec2,
    world_position: Vec2,
) -> impl FnMut(EntityAssertions<'_, F>) -> EntityAssertions<'_, F>
where
    F: EntityFilter,
{
    move |e| {
        e.has(|c: &Camera2D| {
            assert_approx_eq!(
                c.world_position(surface_size, surface_position),
                world_position
            );
        })
    }
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_one_target() {
    let missing_target_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CAMERA, TARGET1))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .with_update::<With<Camera2D>, _>(|c: &mut Camera2D| c.target_keys[0] = TARGET2)
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#not_empty2"))
        .with_update::<With<Camera2D>, _>(|c: &mut Camera2D| c.target_keys[0] = missing_target_key)
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_many_targets() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CAMERA, TARGET1).with_target_key(TARGET2))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#not_empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_transform() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CAMERA, TARGET1))
        .updated()
        .with_component::<With<Camera2D>, _>(|| {
            Transform2D::new()
                .with_position(Vec2::ONE * 0.5)
                .with_size(Vec2::ONE * 2.)
                .with_rotation(FRAC_PI_2)
        })
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty_offset1"))
        .assert::<With<Camera2D>>(
            1,
            assert_position(Size::new(800, 600), Vec2::ZERO, Vec2::new(1.5, 11. / 6.)),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(
                Size::new(800, 600),
                Vec2::new(800., 600.),
                Vec2::new(-0.5, -5. / 6.),
            ),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(Size::new(600, 800), Vec2::ZERO, Vec2::new(11. / 6., 1.5)),
        )
        .assert::<With<Camera2D>>(
            1,
            assert_position(
                Size::new(600, 800),
                Vec2::new(600., 800.),
                Vec2::new(-5. / 6., -0.5),
            ),
        )
        .with_deleted_components::<With<Camera2D>, Transform2D>()
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"));
}

#[modor_test(disabled(macos, android, wasm))]
fn recreate_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CAMERA, TARGET1))
        .updated()
        .with_deleted_entities::<With<Camera2D>>()
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"))
        .with_entity(Camera2D::new(CAMERA, TARGET1))
        .updated()
        .assert::<With<Target1>>(1, is_same("camera#not_empty1"))
        .assert::<With<Target2>>(1, is_same("camera#empty2"));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_entity(target1())
        .child_entity(target2())
        .child_component(Material::new(MATERIAL).with_color(Color::BLUE))
        .child_entity(model())
}

fn target1() -> impl BuiltEntity {
    let texture_key = ResKey::unique("target-1");
    EntityBuilder::new()
        .component(RenderTarget::new(TARGET1))
        .component(Texture::from_size(texture_key, Size::new(30, 20)))
        .component(TextureBuffer::default())
        .component(Target1)
}

fn target2() -> impl BuiltEntity {
    let texture_key = ResKey::unique("target-2");
    EntityBuilder::new()
        .component(RenderTarget::new(TARGET2))
        .component(Texture::from_size(texture_key, Size::new(20, 30)))
        .component(TextureBuffer::default())
        .component(Target2)
}

fn model() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(
            Transform2D::new()
                .with_position(Vec2::ONE * 0.25)
                .with_size(Vec2::ONE * 0.5),
        )
        .component(Model::rectangle(MATERIAL, CAMERA))
}

const TARGET1: ResKey<RenderTarget> = ResKey::new("1");
const TARGET2: ResKey<RenderTarget> = ResKey::new("2");
const MATERIAL: ResKey<Material> = ResKey::new("main");
const CAMERA: ResKey<Camera2D> = ResKey::new("main");

#[derive(SingletonComponent, NoSystem)]
struct Target1;

#[derive(SingletonComponent, NoSystem)]
struct Target2;
