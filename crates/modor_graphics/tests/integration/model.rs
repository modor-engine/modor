use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    Camera2D, Color, GraphicsModule, Material, Model, RenderTarget, Size, Texture, TextureBuffer,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::IntoResourceKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_default() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().with(Model::rectangle(MaterialKey::OpaqueBlue)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_without_transform() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_opaque_with_one_camera() {
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default),
            ),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = CameraKey::Offset.into_key())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#other_camera_opaque"))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = CameraKey::Missing.into_key())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_transparent_with_one_camera() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().with(
            Model::rectangle(MaterialKey::TransparentBlue).with_camera_key(CameraKey::Default),
        ))
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        )
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = CameraKey::Offset.into_key())
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("model#other_camera_transparent", 1))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = CameraKey::Missing.into_key())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_many_cameras() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue)
                    .with_camera_key(CameraKey::Default)
                    .with_camera_key(CameraKey::Offset),
            ),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#many_cameras_opaque"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_material() {
    // test transition between opaque-opaque, transparent-opaque, opaque-transparent, ...
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform()
                .with(Model::rectangle(MaterialKey::OpaqueRed).with_camera_key(CameraKey::Default)),
        )
        .updated()
        .with_update::<With<Model>, _>(|m: &mut Model| {
            m.material_key = MaterialKey::OpaqueBlue.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .updated()
        .with_update::<With<Model>, _>(|m: &mut Model| {
            m.material_key = MaterialKey::TransparentRed.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("model#one_camera_transparent_red", 1))
        .with_update::<With<Model>, _>(|m: &mut Model| {
            m.material_key = MaterialKey::TransparentBlue.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        )
        .with_update::<With<Model>, _>(|m: &mut Model| {
            m.material_key = MaterialKey::OpaqueBlue.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .with_update::<With<Model>, _>(|m: &mut Model| {
            m.material_key = MaterialKey::Missing.into_key();
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(MaterialKey::OpaqueRed, Vec2::ONE * 0.2).with(ToDelete))
        .with_entity(rectangle(MaterialKey::OpaqueBlue, Vec2::ONE * 0.4).with(ToDelete))
        .with_entity(rectangle(MaterialKey::OpaqueBlue, Vec2::ZERO).with(BlankComponent))
        .with_entity(rectangle(MaterialKey::TransparentBlue, -Vec2::ONE * 0.2).with(ToDelete))
        .updated()
        .with_deleted_entities::<With<ToDelete>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"));
}

#[modor_test]
fn create_without_graphics_module() {
    App::new()
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default),
            ),
        )
        .updated();
}

#[modor_test(disabled(macos, android, wasm))]
fn create_graphics_module() {
    App::new()
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default),
            ),
        )
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"));
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_graphics_module_with_opaque_model() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default),
            ),
        )
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"));
}

#[modor_test(disabled(macos, android, wasm))]
fn replace_graphics_module_with_transparent_model() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().with(
            Model::rectangle(MaterialKey::TransparentBlue).with_camera_key(CameraKey::Default),
        ))
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        );
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module_with_opaque_model() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform().with(
                Model::rectangle(MaterialKey::OpaqueBlue).with_camera_key(CameraKey::Default),
            ),
        )
        .updated()
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module_with_transparent_model() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().with(
            Model::rectangle(MaterialKey::TransparentBlue).with_camera_key(CameraKey::Default),
        ))
        .updated()
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        );
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(target())
        .with_child(opaque_blue_material())
        .with_child(opaque_red_material())
        .with_child(transparent_blue_material())
        .with_child(transparent_red_material())
        .with_child(default_camera())
        .with_child(offset_camera())
}

fn target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Texture::from_size(TargetTextureKey, Size::new(30, 20)))
        .with(TextureBuffer::default())
}

fn opaque_blue_material() -> Material {
    Material::new(MaterialKey::OpaqueBlue).with_color(Color::BLUE)
}

fn opaque_red_material() -> Material {
    Material::new(MaterialKey::OpaqueRed).with_color(Color::RED)
}

fn transparent_blue_material() -> Material {
    Material::new(MaterialKey::TransparentBlue).with_color(Color::BLUE.with_alpha(0.5))
}

fn transparent_red_material() -> Material {
    Material::new(MaterialKey::TransparentRed).with_color(Color::RED.with_alpha(0.5))
}

fn default_camera() -> Camera2D {
    Camera2D::new(CameraKey::Default).with_target_key(TargetKey)
}

fn offset_camera() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Camera2D::new(CameraKey::Offset).with_target_key(TargetKey))
        .with(Transform2D::new().with_position(Vec2::new(0.5, 0.5)))
}

fn rectangle(material_key: MaterialKey, position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position))
        .with(Model::rectangle(material_key).with_camera_key(CameraKey::Default))
}

fn transform() -> impl BuiltEntity {
    EntityBuilder::new().with(Transform2D::new())
}

#[derive(Component, NoSystem)]
struct ToDelete;

#[derive(Component, NoSystem)]
struct BlankComponent; // used to control insertion order of instances

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetTextureKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    OpaqueBlue,
    OpaqueRed,
    TransparentBlue,
    TransparentRed,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CameraKey {
    Default,
    Offset,
    Missing,
}