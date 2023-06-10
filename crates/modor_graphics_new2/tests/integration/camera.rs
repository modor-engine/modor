use crate::assert_exact_texture;
use modor::{App, BuiltEntity, EntityAssertions, EntityBuilder, EntityFilter, With};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_internal::assert_approx_eq;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::IntoResourceKey;
use std::f32::consts::FRAC_PI_2;

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CameraKey))
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#empty2"))
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
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CameraKey).with_target_key(TargetKey::First))
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#not_empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#empty2"))
        .with_update::<With<Camera2D>, _>(|c: &mut Camera2D| {
            c.target_keys[0] = TargetKey::Second.into_key();
        })
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#not_empty2"))
        .with_update::<With<Camera2D>, _>(|c: &mut Camera2D| {
            c.target_keys[0] = TargetKey::Missing.into_key();
        })
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_many_targets() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(
            Camera2D::new(CameraKey)
                .with_target_key(TargetKey::First)
                .with_target_key(TargetKey::Second),
        )
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#not_empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#not_empty2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_transform() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CameraKey).with_target_key(TargetKey::First))
        .updated()
        .with_component::<With<Camera2D>, _>(|| {
            Transform2D::new()
                .with_position(Vec2::ONE * 0.5)
                .with_size(Vec2::ONE * 2.)
                .with_rotation(FRAC_PI_2)
        })
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#not_empty_offset1"))
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
        .assert::<With<Target1>>(1, assert_exact_texture("camera#not_empty1"));
}

#[modor_test(disabled(macos, android, wasm))]
fn recreate_entity() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(resources())
        .with_entity(Camera2D::new(CameraKey).with_target_key(TargetKey::First))
        .updated()
        .with_deleted_entities::<With<Camera2D>>()
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#empty2"))
        .with_entity(Camera2D::new(CameraKey).with_target_key(TargetKey::First))
        .updated()
        .assert::<With<Target1>>(1, assert_exact_texture("camera#not_empty1"))
        .assert::<With<Target2>>(1, assert_exact_texture("camera#empty2"));
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(target1())
        .with_child(target2())
        .with_child(Material::new(MaterialKey).with_color(Color::BLUE))
        .with_child(model())
}

fn target1() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::First))
        .with(Texture::from_size(
            TargetTextureKey::First,
            Size::new(30, 20),
        ))
        .with(TextureBuffer::default())
        .with(Target1)
}

fn target2() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::Second))
        .with(Texture::from_size(
            TargetTextureKey::Second,
            Size::new(20, 30),
        ))
        .with(TextureBuffer::default())
        .with(Target2)
}

fn model() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::ONE * 0.25)
                .with_size(Vec2::ONE * 0.5),
        )
        .with(Model::rectangle(MaterialKey).with_camera_key(CameraKey))
}

#[derive(SingletonComponent, NoSystem)]
struct Target1;

#[derive(SingletonComponent, NoSystem)]
struct Target2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TargetKey {
    First,
    Second,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TargetTextureKey {
    First,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct MaterialKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;
