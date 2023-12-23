use modor::{App, BuiltEntity, EntityBuilder, With};
use modor_graphics::testing::{has_component_diff, is_same};
use modor_graphics::{
    instance_2d, texture_target, Color, Default2DMaterial, GraphicsModule, MaterialSource,
    NoInstanceData, Shader, ShaderSource, Size, Texture, TextureBuffer, ZIndex2D,
    TEXTURE_CAMERAS_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::testing::wait_resource_loading;
use modor_resources::ResKey;

#[modor_test(disabled(macos, android, wasm))]
fn create_from_path() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(custom_instance())
        .with_entity(Shader::from_path::<NoInstanceData>(
            CUSTOM_SHADER,
            "../tests/assets/red.wgsl",
            false,
        ))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#empty"))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_from_string() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_string::<NoInstanceData>(
            CUSTOM_SHADER,
            CUSTOM_SHADER_CODE,
            false,
        ))
        .with_entity(custom_instance())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#custom"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_with_alpha_replacing() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_path::<NoInstanceData>(
            CUSTOM_SHADER,
            "../tests/assets/color.wgsl",
            true,
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(colored_instance(Vec2::ONE, 0, Color::RED))
        .with_entity(colored_instance(
            Vec2::ONE * 0.5,
            1,
            Color::GREEN.with_alpha(0.2),
        ))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#alpha_replacing"));
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
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_path::<NoInstanceData>(
            CUSTOM_SHADER,
            format!("../tests/assets/{shader_filename}"),
            false,
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(custom_instance())
        .updated()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("shader#empty"));
}

#[modor_test(disabled(macos, android, wasm))]
fn set_source() {
    #[derive(Component, NoSystem)]
    struct CustomShader;
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(
            EntityBuilder::new()
                .component(Shader::from_path::<NoInstanceData>(
                    CUSTOM_SHADER,
                    "../tests/assets/red.wgsl",
                    false,
                ))
                .component(CustomShader),
        )
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(custom_instance())
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
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(Shader::from_path::<NoInstanceData>(
            CUSTOM_SHADER,
            "../tests/assets/red.wgsl",
            false,
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(custom_instance())
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
    cases(default = "\"default\", false", ellipse = "\"ellipse\", true")
)]
fn use_builtin(label: &str, is_ellipse: bool) {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(textures())
        .updated_until_all::<(), Texture>(Some(100), wait_resource_loading)
        .with_entity(instance(is_ellipse))
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
}

fn instance(is_ellipse: bool) -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), Default2DMaterial::new())
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.8, 0.5))
        .updated(|m: &mut Default2DMaterial| m.color = Color::GRAY)
        .updated(|m: &mut Default2DMaterial| m.texture_key = Some(BACK_TEXTURE))
        .updated(|m: &mut Default2DMaterial| m.texture_position = Vec2::new(0.5, 0.))
        .updated(|m: &mut Default2DMaterial| m.texture_size = Vec2::new(0.5, 1.))
        .updated(|m: &mut Default2DMaterial| m.is_ellipse = is_ellipse)
}

fn custom_instance() -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), CustomMaterial::default())
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.8, 0.5))
}

fn colored_instance(size: Vec2, z: u16, color: Color) -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), CustomMaterial::default())
        .updated(|m: &mut CustomMaterial| m.color = color)
        .updated(|t: &mut Transform2D| t.size = size)
        .component(ZIndex2D::from(z))
}

#[derive(Component, NoSystem)]
struct CustomMaterial {
    color: Color,
}

impl Default for CustomMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
        }
    }
}

impl MaterialSource for CustomMaterial {
    type Data = [f32; 4];
    type InstanceData = NoInstanceData;

    fn data(&self) -> Self::Data {
        self.color.into()
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        CUSTOM_SHADER
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

const BACK_TEXTURE: ResKey<Texture> = ResKey::new("background");
const CUSTOM_SHADER: ResKey<Shader> = ResKey::new("custom");
const CUSTOM_SHADER_CODE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/red.wgsl"
));
const CUSTOM2_SHADER_CODE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/green.wgsl"
));
