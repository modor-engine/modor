use crate::gpu_data::uniform::Uniform;
use crate::resources::mesh::MeshKey;
use crate::resources::shader::ShaderKey;
use crate::resources::texture::TextureRegistry;
use crate::{
    Color, GraphicsModule, IntoResourceKey, Resource, ResourceKey, ResourceRegistry, ResourceState,
    Texture,
};
use modor::{Query, Single, SingleMut};

pub(crate) type MaterialRegistry = ResourceRegistry<Material>;

// TODO: add texture position and size params

// performance impact if lots of materials
#[must_use]
#[derive(Debug)]
pub struct Material {
    pub color: Color,
    pub texture_key: Option<ResourceKey>,
    key: ResourceKey,
    shader_key: ResourceKey,
    mesh_key: ResourceKey,
    uniform: Option<Uniform<MaterialData>>,
    is_transparent: bool,
    old_is_transparent: bool,
}

#[component]
impl Material {
    const BINDING: u32 = 0;

    pub fn rectangle(key: impl IntoResourceKey) -> Self {
        Self::new(key, ShaderKey::Rectangle)
    }

    pub fn ellipse(key: impl IntoResourceKey) -> Self {
        Self::new(key, ShaderKey::Ellipse)
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_texture(mut self, key: impl IntoResourceKey) -> Self {
        self.texture_key = Some(key.into_key());
        self
    }

    fn new(key: impl IntoResourceKey, shader_key: ShaderKey) -> Self {
        Self {
            color: Color::WHITE,
            texture_key: None,
            key: key.into_key(),
            shader_key: shader_key.into_key(),
            mesh_key: MeshKey::Rectangle.into_key(),
            uniform: None,
            is_transparent: false,
            old_is_transparent: false,
        }
    }

    #[run]
    fn update_uniform(&mut self, module: Option<Single<'_, GraphicsModule>>) {
        if let Some(module) = module {
            let data = MaterialData {
                color: self.color.into(),
                texture_part_position: [0., 0.],
                texture_part_size: [1., 1.],
                has_texture: 0,
                _padding: [0., 0., 0.],
            };
            if let Some(uniform) = &mut self.uniform {
                if data != **uniform {
                    **uniform = data;
                    uniform.sync(&module);
                }
            } else {
                self.uniform = Some(Uniform::new(
                    data,
                    Self::BINDING,
                    &module.material_bind_group_layout,
                    &format!("material_{:?}", &self.key),
                    &module.device,
                ));
            }
        } else {
            self.uniform = None;
        }
    }

    #[run_after(component(TextureRegistry), component(Texture))]
    fn update_transparency(
        &mut self,
        (mut texture_registry, textures): (SingleMut<'_, TextureRegistry>, Query<'_, &Texture>),
    ) {
        self.old_is_transparent = self.is_transparent;
        if !self.is_transparent {
            self.is_transparent = (self.color.a > 0. && self.color.a < 1.)
                || self
                    .texture_key
                    .as_ref()
                    .and_then(|k| texture_registry.get(k, &textures))
                    .map_or(false, Texture::is_transparent);
        }
    }

    pub(crate) fn is_transparent(&self) -> bool {
        self.is_transparent
    }

    pub(crate) fn is_newly_transparent(&self) -> bool {
        !self.old_is_transparent && self.is_transparent
    }

    pub(crate) fn uniform(&self) -> &Uniform<MaterialData> {
        self.uniform
            .as_ref()
            .expect("internal error: material uniform not initialized")
    }

    pub(crate) fn shader_key(&self) -> &ResourceKey {
        &self.shader_key
    }

    pub(crate) fn mesh_key(&self) -> &ResourceKey {
        &self.mesh_key
    }
}

impl Resource for Material {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.uniform.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct MaterialData {
    pub(crate) color: [f32; 4],
    pub(crate) texture_part_position: [f32; 2],
    pub(crate) texture_part_size: [f32; 2],
    pub(crate) has_texture: u32,
    pub(crate) _padding: [f32; 3],
}
