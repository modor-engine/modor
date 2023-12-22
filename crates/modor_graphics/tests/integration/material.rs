use bytemuck::{Pod, Zeroable};
use modor::{App, Changed, SystemParamWithLifetime, With};
use modor_graphics::testing::is_same;
use modor_graphics::{
    instance_2d, instance_2d_with_key, material, texture_target, Color, GraphicsModule,
    InstanceData, InstanceGroup2D, InstanceRendering2D, Material, MaterialSource, NoInstanceData,
    Shader, Size, Texture, TextureBuffer, TEXTURE_CAMERAS_2D,
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
        .with_entity(Shader::from_path::<NoInstanceData>(
            COLOR_SHADER,
            "../tests/assets/color.wgsl",
        ))
        .with_entity(Shader::from_path::<NoInstanceData>(
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
        .with_entity(Shader::from_path::<NoInstanceData>(
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
        .with_entity(Shader::from_path::<NoInstanceData>(
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

#[modor_test(
    disabled(macos, android, wasm),
    cases(opaque = "false", transparent = "true")
)]
fn create_material_with_instance_data(is_transparent: bool) {
    let instance_material_key = ResKey::new("instance");
    let simple_material_key = ResKey::new("simple");
    let group_key = ResKey::new("group");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(texture_target(1, Size::new(30, 20), true))
        .with_entity(texture_target(2, Size::new(30, 20), true))
        .with_entity(Shader::from_path::<CustomInstanceData>(
            INSTANCE_SHADER,
            "../tests/assets/instance.wgsl",
        ))
        .with_entity(Shader::from_path::<NoInstanceData>(
            COLOR_SHADER,
            "../tests/assets/color.wgsl",
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<CustomInstanceMaterial>(instance_material_key))
        .with_entity(material::<CustomMaterial>(simple_material_key))
        .with_update::<(), _>(|m: &mut CustomInstanceMaterial| m.is_transparent = is_transparent)
        .with_entity(instance_2d_with_key(
            group_key,
            TEXTURE_CAMERAS_2D.get(0),
            instance_material_key,
        ))
        .with_entity(InstanceRendering2D::new(
            group_key,
            TEXTURE_CAMERAS_2D.get(1),
            instance_material_key,
        ))
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(2), simple_material_key))
        .updated()
        .assert::<With<TextureBuffer>>(3, is_same("material#red"))
        .with_component::<With<InstanceGroup2D>, _>(|| CustomColor(Color::GREEN))
        .with_update::<(), CustomMaterial>(|m| m.color = Color::GREEN)
        .updated()
        .assert::<With<TextureBuffer>>(3, is_same("material#green"))
        .with_update::<(), _>(|i: &mut CustomColor| i.0 = Color::BLUE)
        .with_update::<(), CustomMaterial>(|m| m.color = Color::BLUE)
        .updated()
        .updated()
        .assert::<With<TextureBuffer>>(3, is_same("material#blue"));
}

#[modor_test(disabled(macos, android, wasm))]
fn create_material_with_incorrect_instance_data_type() {
    let material_key = ResKey::new("material");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Shader::from_path::<OtherCustomInstanceData>(
            INSTANCE_SHADER,
            "../tests/assets/instance.wgsl",
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<CustomInstanceMaterial>(material_key))
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(0), material_key))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#empty"))
        .assert::<With<CustomInstanceMaterial>>(1, |e| {
            e.has(|m: &Material| assert!(matches!(m.state(), ResourceState::Error(_))))
        });
}

#[modor_test(disabled(macos, android, wasm))]
fn delete_and_recreate_graphics_module_with_instance_data() {
    let material_key = ResKey::new("material");
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(texture_target(0, Size::new(30, 20), true))
        .with_entity(Shader::from_path::<CustomInstanceData>(
            INSTANCE_SHADER,
            "../tests/assets/instance.wgsl",
        ))
        .updated_until_all::<(), Shader>(Some(100), wait_resource_loading)
        .with_entity(material::<CustomInstanceMaterial>(material_key))
        .with_entity(instance_2d(TEXTURE_CAMERAS_2D.get(0), material_key))
        .with_component::<With<InstanceGroup2D>, _>(|| CustomColor(Color::GREEN))
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#green"))
        .with_deleted_entities::<With<GraphicsModule>>()
        .updated()
        .with_entity(modor_graphics::module())
        .updated()
        .assert::<With<TextureBuffer>>(1, is_same("material#green"));
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
    type InstanceData = NoInstanceData;

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
#[derive(Clone, Copy, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
struct CustomMaterialData {
    color: [f32; 4],
}

#[derive(Component, NoSystem, Default)]
struct EmptyMaterial;

impl MaterialSource for EmptyMaterial {
    type Data = [u32; 0];
    type InstanceData = NoInstanceData;

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

#[derive(Component, NoSystem, Default)]
struct CustomInstanceMaterial {
    is_transparent: bool,
}

impl MaterialSource for CustomInstanceMaterial {
    type Data = [u32; 0];
    type InstanceData = CustomInstanceData;

    fn data(&self) -> Self::Data {
        []
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        INSTANCE_SHADER
    }

    fn is_transparent(&self) -> bool {
        self.is_transparent
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct CustomInstanceData {
    color: [f32; 4],
}

impl Default for CustomInstanceData {
    fn default() -> Self {
        Self {
            color: Color::RED.into(),
        }
    }
}

impl InstanceData for CustomInstanceData {
    type Query = &'static CustomColor;
    type UpdateFilter = Changed<CustomColor>;

    fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
        Self {
            color: item.0.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct OtherCustomInstanceData {
    color: [f32; 4],
}

impl Default for OtherCustomInstanceData {
    fn default() -> Self {
        Self {
            color: Color::RED.into(),
        }
    }
}

impl InstanceData for OtherCustomInstanceData {
    type Query = &'static CustomColor;
    type UpdateFilter = Changed<CustomColor>;

    fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
        Self {
            color: item.0.into(),
        }
    }
}

#[derive(Component, NoSystem)]
struct CustomColor(Color);

const COLOR_SHADER: ResKey<Shader> = ResKey::new("color");
const TEXTURE_SHADER: ResKey<Shader> = ResKey::new("textures");
const INSTANCE_SHADER: ResKey<Shader> = ResKey::new("instance");
const TEXTURE: ResKey<Texture> = ResKey::new("texture");
const CYAN_TEXTURE: ResKey<Texture> = ResKey::new("cyan");
const MAGENTA_TEXTURE: ResKey<Texture> = ResKey::new("magenta");
