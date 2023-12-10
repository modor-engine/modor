use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    instance_2d, texture_target, Color, GraphicsModule, Material, Shader, ShaderSource, Size,
    Texture, TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_from_path() {
    let shader_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(instance(shader_key))
        .with_entity(Shader::from_path(shader_key, "../tests/assets/custom.wgsl"))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#empty"))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_string() {
    let shader_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_string(shader_key, CUSTOM_SHADER_CODE))
        .with_entity(instance(shader_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"));
}

#[modor_test(
    disabled(macos, android, wasm),
    cases(
        invalid_syntax = "\"invalid-syntax.wgsl\"",
        invalid_binding = "\"invalid-binding.wgsl\"",
        invalid_camera = "\"invalid-camera.wgsl\"",
    )
)]
fn create_with(shader_filename: &str) {
    let shader_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_path(
            shader_key,
            format!("../tests/assets/{shader_filename}"),
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(instance(shader_key))
        .updated()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn set_source() {
    #[derive(Component, NoSystem)]
    struct CustomShader;
    let shader_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(
            EntityBuilder::new()
                .component(Shader::from_path(shader_key, "../tests/assets/custom.wgsl"))
                .component(CustomShader),
        )
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(instance(shader_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"))
        .with_update::<With<CustomShader>, _>(|s: &mut Shader| {
            s.set_source(ShaderSource::String(CUSTOM2_SHADER_CODE));
        })
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom2"));
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module() {
    let shader_key = ResKey::new("custom");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_path(shader_key, "../tests/assets/custom.wgsl"))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(instance(shader_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"))
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .with_entity(modor_graphics::module())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"));
}

#[modor_test(
    disabled(macos, android, wasm),
    cases(
        default = "\"default\", modor_graphics::DEFAULT_SHADER",
        ellipse = "\"ellipse\", modor_graphics::ELLIPSE_SHADER"
    )
)]
fn use_builtin(label: &str, shader_key: ResKey<Shader>) {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(instance(shader_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, has_component_diff(&format!("shader#{label}"), 50, 2));
}

fn textures() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Texture::from_path(
            BACK_TEXTURE,
            "../tests/assets/opaque-texture.png",
        ))
        .with(|t| t.is_smooth = false)
        .with(|t| t.is_repeated = false)
        .child_component(Texture::from_path(
            FRONT_TEXTURE,
            "../tests/assets/no-border.png",
        ))
        .with(|t| t.is_smooth = false)
        .with(|t| t.is_repeated = false)
}

fn instance(shader_key: ResKey<Shader>) -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), None)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.8, 0.5))
        .updated(|m: &mut Material| m.color = Color::GRAY)
        .updated(|m: &mut Material| m.texture_key = Some(BACK_TEXTURE))
        .updated(|m: &mut Material| m.texture_position = Vec2::new(0.5, 0.))
        .updated(|m: &mut Material| m.texture_size = Vec2::new(0.5, 1.))
        .updated(|m: &mut Material| m.front_color = Color::RED)
        .updated(|m: &mut Material| m.front_texture_key = Some(FRONT_TEXTURE))
        .updated(|m: &mut Material| m.shader_key = shader_key)
}

const BACK_TEXTURE: ResKey<Texture> = ResKey::new("background");
const FRONT_TEXTURE: ResKey<Texture> = ResKey::new("foreground");
const CUSTOM_SHADER_CODE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/custom.wgsl"
));
const CUSTOM2_SHADER_CODE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/custom2.wgsl"
));
