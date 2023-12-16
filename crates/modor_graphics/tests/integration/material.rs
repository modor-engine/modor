use modor::{App, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    instance_2d, material, texture_target, Color, Material, MaterialSource, Shader, Size, Texture,
    TextureBuffer, TEXTURE_CAMERAS_2D,
};
use modor_resources::testing::wait_resource_loading;
use modor_resources::{ResKey, Resource, ResourceState};

#[modor_test(disabled(macos, android, wasm))]
fn update_properties() {
    let material_key = ResKey::new("material");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Texture::from_buffer(
            CYAN_TEXTURE,
            Size::new(1, 1),
            vec![0, 255, 255, 255],
        ))
        .with_entity(Texture::from_buffer(
            MAGENTA_TEXTURE,
            Size::new(1, 1),
            vec![255, 0, 255, 255],
        ))
        .with_entity(Shader::from_path(
            COLOR_SHADER,
            "../tests/assets/color.wgsl",
        ))
        .with_entity(Shader::from_path(
            TEXTURE_SHADER,
            "../tests/assets/textures.wgsl",
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<CustomMaterial>(material_key))
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(0), material_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#red"))
        .with_update::<(), CustomMaterial>(|m| m.color = Color::GREEN)
        .updated()
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#green"))
        .with_update::<(), CustomMaterial>(|m| m.shader_key = TEXTURE_SHADER)
        .with_update::<(), CustomMaterial>(|m| m.texture_keys = vec![MAGENTA_TEXTURE, CYAN_TEXTURE])
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#blue"));
}

#[modor_test(disabled(macos, android, wasm))]
fn use_material_with_empty_data() {
    let material_key = ResKey::new("material");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Texture::from_buffer(
            CYAN_TEXTURE,
            Size::new(1, 1),
            vec![0, 255, 255, 255],
        ))
        .with_entity(Texture::from_buffer(
            MAGENTA_TEXTURE,
            Size::new(1, 1),
            vec![255, 0, 255, 255],
        ))
        .with_entity(Shader::from_path(
            TEXTURE_SHADER,
            "../tests/assets/textures.wgsl",
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<EmptyMaterial>(material_key))
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(0), material_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#blue"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_material_with_texture_count_different_than_shader() {
    let material_key = ResKey::new("material");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Shader::from_path(
            COLOR_SHADER,
            "../tests/assets/color.wgsl",
        ))
        .with_entity(Texture::from_size(TEXTURE, Size::new(1, 1)))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<CustomMaterial>(material_key))
        .with_update::<(), CustomMaterial>(|m| m.texture_keys = vec![TEXTURE])
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(0), material_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"))
        .assert::<With<CustomMaterial>>(1, |e| {
            e.has(|m: &Material| assert!(matches!(m.state(), ResourceState::Error(_))))
        });
}

#[derive(Component)]
struct CustomMaterial {
    color: Color,
    shader_key: ResKey<Shader>,
    texture_keys: Vec<ResKey<Texture>>,
}

#[systems]
impl CustomMaterial {}

impl Default for CustomMaterial {
    fn default() -> Self {
        Self {
            color: Color::RED,
            shader_key: COLOR_SHADER,
            texture_keys: vec![],
        }
    }
}

impl MaterialSource for CustomMaterial {
    type Data = CustomMaterialData;

    fn data(&self) -> Self::Data {
        CustomMaterialData {
            color: self.color.into(),
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        self.texture_keys.clone()
    }

    fn shader_key(&self) -> ResKey<Shader> {
        self.shader_key
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
struct CustomMaterialData {
    color: [f32; 4],
}

#[derive(Component, NoSystem, Default)]
struct EmptyMaterial;

impl MaterialSource for EmptyMaterial {
    type Data = [u32; 0];

    fn data(&self) -> Self::Data {
        []
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![CYAN_TEXTURE, MAGENTA_TEXTURE]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        TEXTURE_SHADER
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

const COLOR_SHADER: ResKey<Shader> = ResKey::new("color");
const TEXTURE_SHADER: ResKey<Shader> = ResKey::new("textures");
const TEXTURE: ResKey<Texture> = ResKey::new("texture");
const CYAN_TEXTURE: ResKey<Texture> = ResKey::new("cyan");
const MAGENTA_TEXTURE: ResKey<Texture> = ResKey::new("magenta");

// use modor::{App, BuiltEntity, EntityBuilder, With};
// use modor_graphics::testing::{has_component_diff, has_pixel_diff, is_same};
// use modor_graphics::{
//     instance_2d, texture_target, Color, Material, RenderTarget, Size, Texture, TextureBuffer,
//     ELLIPSE_SHADER, TEXTURE_CAMERAS_2D,
// };
// use modor_math::Vec2;
// use modor_physics::Transform2D;
// use modor_resources::ResKey;
//
// #[modor_test(disabled(macos, android, wasm))]
// fn create_default() {
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(Material::new(MATERIAL))
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#color_white"));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_color() {
//     let mut material = Material::new(MATERIAL);
//     material.color = Color::GREEN;
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#color_green"))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.color = Color::RED)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_texture() {
//     let missing_texture_key = ResKey::new("missing");
//     let mut material = Material::new(MATERIAL);
//     material.texture_key = Some(OPAQUE_TEXTURE);
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture"))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_key = Some(TRANSPARENT_TEXTURE))
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_component_diff("material#transparent_texture", 1, 1))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_key = None)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#color_white"))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_key = Some(missing_texture_key))
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_color_and_texture() {
//     let mut material = Material::new(MATERIAL);
//     material.color = Color::RED;
//     material.texture_key = Some(OPAQUE_TEXTURE);
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#opaque_texture_red"))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_key = Some(TRANSPARENT_TEXTURE))
//         .updated()
//         .assert::<With<TextureBuffer>>(
//             1,
//             has_component_diff("material#transparent_texture_red", 1, 1),
//         )
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_key = None)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#color_red"));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_cropped_texture() {
//     let mut material = Material::new(MATERIAL);
//     material.texture_key = Some(OPAQUE_TEXTURE);
//     material.texture_position = Vec2::new(0.5, 0.);
//     material.texture_size = Vec2::new(0.5, 1.);
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_component_diff("material#cropped_texture", 1, 1))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_position = Vec2::ZERO)
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.texture_size = Vec2::ONE)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_component_diff("material#opaque_texture", 1, 1));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_front_texture() {
//     let missing_texture_key = ResKey::new("missing");
//     let mut material = Material::new(MATERIAL);
//     material.front_texture_key = Some(OPAQUE_TEXTURE);
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.front_texture_key = Some(TRANSPARENT_TEXTURE))
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture", 10))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.front_texture_key = None)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#color_white", 10))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.front_texture_key = Some(missing_texture_key))
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn configure_front_color_and_texture() {
//     let mut material = Material::new(MATERIAL);
//     material.front_texture_key = Some(TRANSPARENT_TEXTURE);
//     material.front_color = Color::RED;
//     material.color = Color::GREEN;
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_red", 10))
//         .with_update::<(), _>(|m: &mut Default2DMaterial| m.front_color = Color::BLUE)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#front_texture_blue", 10));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn create_with_not_default_shader() {
//     let mut material = Material::new(MATERIAL);
//     material.color = Color::GREEN;
//     material.texture_key = Some(OPAQUE_TEXTURE);
//     material.texture_position = Vec2::new(0.5, 0.);
//     material.texture_size = Vec2::new(0.5, 1.);
//     material.front_color = Color::RED;
//     material.front_texture_key = Some(TRANSPARENT_TEXTURE);
//     material.shader_key = ELLIPSE_SHADER;
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .assert::<With<TextureBuffer>>(1, has_pixel_diff("material#ellipse", 10));
// }
//
// #[modor_test(disabled(macos, android, wasm))]
// fn delete_entity() {
//     let mut material = Material::new(MATERIAL);
//     material.color = Color::GREEN;
//     App::new()
//         .with_entity(modor_graphics::module())
//         .with_entity(resources())
//         .with_entity(material)
//         .updated()
//         .with_deleted_entities::<With<Material>>()
//         .updated()
//         .assert::<With<TextureBuffer>>(1, is_same("material#empty"));
// }
//
// fn resources() -> impl BuiltEntity {
//     EntityBuilder::new()
//         .child_entity(
//             texture_target(0, Size::new(30, 20), true)
//                 .updated(|t: &mut RenderTarget| t.background_color = Color::DARK_GRAY),
//         )
//         .child_component(opaque_texture())
//         .child_component(transparent_texture())
//         .child_entity(rectangle())
// }
//
// fn rectangle() -> impl BuiltEntity {
//     instance_2d::<Default2DMaterial>(TEXTURE_CAMERAS_2D.get(0), Some(MATERIAL))
//         .updated(|t: &mut Transform2D| t.size = Vec2::new(0.8, 0.5))
// }
//
// fn opaque_texture() -> Texture {
//     let white_pixel = [255, 255, 255, 255];
//     let gray_pixel = [128, 128, 128, 255];
//     let texture = [
//         [white_pixel, white_pixel, gray_pixel, gray_pixel],
//         [white_pixel, white_pixel, gray_pixel, gray_pixel],
//         [gray_pixel, gray_pixel, white_pixel, white_pixel],
//         [gray_pixel, gray_pixel, white_pixel, white_pixel],
//     ]
//     .into_iter()
//     .flat_map(|l| l.into_iter().flatten())
//     .collect();
//     let mut texture = Texture::from_buffer(OPAQUE_TEXTURE, Size::new(4, 4), texture);
//     texture.is_smooth = false;
//     texture
// }
//
// fn transparent_texture() -> Texture {
//     let border_pixel = [0, 0, 0, 128];
//     let center_pixel = [255, 255, 255, 255];
//     let texture = [
//         [border_pixel, border_pixel, border_pixel, border_pixel],
//         [border_pixel, center_pixel, center_pixel, border_pixel],
//         [border_pixel, center_pixel, center_pixel, border_pixel],
//         [border_pixel, border_pixel, border_pixel, border_pixel],
//     ]
//     .into_iter()
//     .flat_map(|l| l.into_iter().flatten())
//     .collect();
//     let mut texture = Texture::from_buffer(TRANSPARENT_TEXTURE, Size::new(4, 4), texture);
//     texture.is_smooth = false;
//     texture
// }
//
// const OPAQUE_TEXTURE: ResKey<Texture> = ResKey::new("opaque");
// const TRANSPARENT_TEXTURE: ResKey<Texture> = ResKey::new("transparent");
// const MATERIAL: ResKey<Material> = ResKey::new("main");
