use log::LevelFilter;
use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    texture_target, Camera2D, Color, GraphicsModule, Material, Model, Size, TextureBuffer,
    TEXTURE_CAMERAS_2D, TEXTURE_TARGETS,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_hidden() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().component(Model::hidden_rectangle(OPAQUE_BLUE_MATERIAL)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_without_transform() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_opaque_with_one_camera() {
    let missing_camera_key = ResKey::new("missing");
    App::new()
        .with_log_level(LevelFilter::Trace)
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA)))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = OFFSET_CAMERA)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#other_camera_opaque"))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = missing_camera_key)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_transparent_with_one_camera() {
    let missing_camera_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform().component(Model::rectangle(TRANSPARENT_BLUE_MATERIAL, DEFAULT_CAMERA)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        )
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = OFFSET_CAMERA)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("model#other_camera_transparent", 1))
        .with_update::<(), _>(|m: &mut Model| m.camera_keys[0] = missing_camera_key)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_many_cameras() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(
            transform()
                .component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA))
                .with(|c| c.camera_keys.push(OFFSET_CAMERA)),
        )
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#many_cameras_opaque"));
}

#[modor_test(disabled(macos, android, wasm))]
fn configure_material() {
    // test transition between opaque-opaque, transparent-opaque, opaque-transparent, ...
    let missing_material_key = ResKey::new("missing");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(transform().component(Model::rectangle(OPAQUE_RED_MATERIAL, DEFAULT_CAMERA)))
        .updated()
        .with_update::<With<Model>, _>(|m: &mut Model| m.material_key = OPAQUE_BLUE_MATERIAL)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .updated()
        .with_update::<With<Model>, _>(|m: &mut Model| m.material_key = TRANSPARENT_RED_MATERIAL)
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff("model#one_camera_transparent_red", 1))
        .with_update::<With<Model>, _>(|m: &mut Model| m.material_key = TRANSPARENT_BLUE_MATERIAL)
        .updated()
        .assert::<With<TextureBuffer>>(
            1,
            has_component_diff("model#one_camera_transparent_blue", 1),
        )
        .with_update::<With<Model>, _>(|m: &mut Model| m.material_key = OPAQUE_BLUE_MATERIAL)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"))
        .with_update::<With<Model>, _>(|m: &mut Model| m.material_key = missing_material_key)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_entity() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(resources())
        .with_entity(rectangle(OPAQUE_RED_MATERIAL, Vec2::ONE * 0.2).component(ToDelete))
        .with_entity(rectangle(OPAQUE_BLUE_MATERIAL, Vec2::ONE * 0.4).component(ToDelete))
        .with_entity(rectangle(OPAQUE_BLUE_MATERIAL, Vec2::ZERO).component(BlankComponent))
        .with_entity(rectangle(TRANSPARENT_BLUE_MATERIAL, -Vec2::ONE * 0.2).component(ToDelete))
        .updated()
        .with_deleted_entities::<With<ToDelete>>()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("model#one_camera_opaque"));
}

#[modor_test]
fn create_without_graphics_module() {
    App::new()
        .with_entity(resources())
        .with_entity(transform().component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA)))
        .updated();
}

#[modor_test(disabled(macos, android, wasm))]
fn create_graphics_module() {
    App::new()
        .with_entity(resources())
        .with_entity(transform().component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA)))
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
        .with_entity(transform().component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA)))
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
        .with_entity(
            transform().component(Model::rectangle(TRANSPARENT_BLUE_MATERIAL, DEFAULT_CAMERA)),
        )
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
        .with_entity(transform().component(Model::rectangle(OPAQUE_BLUE_MATERIAL, DEFAULT_CAMERA)))
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
        .with_entity(
            transform().component(Model::rectangle(TRANSPARENT_BLUE_MATERIAL, DEFAULT_CAMERA)),
        )
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
        .child_entity(texture_target(0, Size::new(30, 20), true))
        .child_entity(materials())
        .child_entity(offset_camera())
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Material::new(OPAQUE_BLUE_MATERIAL))
        .with(|m| m.color = Color::BLUE)
        .child_component(Material::new(OPAQUE_RED_MATERIAL))
        .with(|m| m.color = Color::RED)
        .child_component(Material::new(TRANSPARENT_BLUE_MATERIAL))
        .with(|m| m.color = Color::BLUE.with_alpha(0.5))
        .child_component(Material::new(TRANSPARENT_RED_MATERIAL))
        .with(|m| m.color = Color::RED.with_alpha(0.5))
}

fn offset_camera() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Camera2D::new(OFFSET_CAMERA, TEXTURE_TARGETS.get(0)))
        .component(Transform2D::new())
        .with(|t| *t.position = Vec2::new(0.5, 0.5))
}

fn rectangle(material_key: ResKey<Material>, position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| *t.position = position)
        .component(Model::rectangle(material_key, DEFAULT_CAMERA))
}

fn transform() -> impl BuiltEntity {
    EntityBuilder::new().component(Transform2D::new())
}

#[derive(Component, NoSystem)]
struct ToDelete;

#[derive(Component, NoSystem)]
struct BlankComponent; // used to control insertion order of instances

const OPAQUE_BLUE_MATERIAL: ResKey<Material> = ResKey::new("opaque-blue");
const OPAQUE_RED_MATERIAL: ResKey<Material> = ResKey::new("opaque-red");
const TRANSPARENT_BLUE_MATERIAL: ResKey<Material> = ResKey::new("transparent-blue");
const TRANSPARENT_RED_MATERIAL: ResKey<Material> = ResKey::new("transparent-red");
const DEFAULT_CAMERA: ResKey<Camera2D> = TEXTURE_CAMERAS_2D.get(0);
const OFFSET_CAMERA: ResKey<Camera2D> = ResKey::new("offset");
