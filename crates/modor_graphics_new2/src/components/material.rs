use crate::components::shader::ShaderKey;
use crate::components::texture::TextureRegistry;
use crate::gpu_data::uniform::Uniform;
use crate::{Color, Renderer, Texture};
use modor::{Query, Single, SingleMut};
use modor_math::Vec2;
use modor_resources::{IntoResourceKey, Resource, ResourceKey, ResourceRegistry, ResourceState};

pub(crate) type MaterialRegistry = ResourceRegistry<Material>;

// performance impact if lots of materials
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    pub color: Color,
    pub texture_key: Option<ResourceKey>,
    pub texture_position: Vec2,
    pub texture_size: Vec2,
    pub front_texture_key: Option<ResourceKey>,
    pub front_color: Color,
    pub(crate) shader_key: ResourceKey,
    key: ResourceKey,
    uniform: Option<Uniform<MaterialData>>,
    is_transparent: bool,
    old_is_transparent: bool,
    renderer_version: Option<u8>,
}

#[systems]
impl Material {
    const BINDING: u32 = 0;

    pub fn new(key: impl IntoResourceKey) -> Self {
        Self::new_internal(key, ShaderKey::Default)
    }

    pub fn ellipse(key: impl IntoResourceKey) -> Self {
        Self::new_internal(key, ShaderKey::Ellipse)
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_texture(mut self, key: impl IntoResourceKey) -> Self {
        self.texture_key = Some(key.into_key());
        self
    }

    pub fn with_texture_position(mut self, position: Vec2) -> Self {
        self.texture_position = position;
        self
    }

    pub fn with_texture_size(mut self, size: Vec2) -> Self {
        self.texture_size = size;
        self
    }

    pub fn with_front_texture(mut self, key: impl IntoResourceKey) -> Self {
        self.front_texture_key = Some(key.into_key());
        self
    }

    pub fn with_front_color(mut self, color: Color) -> Self {
        self.front_color = color;
        self
    }

    fn new_internal(key: impl IntoResourceKey, shader_key: ShaderKey) -> Self {
        Self {
            color: Color::WHITE,
            texture_key: None,
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            front_texture_key: None,
            front_color: Color::BLACK,
            key: key.into_key(),
            shader_key: shader_key.into_key(),
            uniform: None,
            is_transparent: false,
            old_is_transparent: false,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer))]
    fn update_uniform(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.uniform = None;
        }
        if let Some(context) = state.context() {
            let data = MaterialData {
                color: self.color.into(),
                texture_part_position: [self.texture_position.x, self.texture_position.y],
                texture_part_size: [self.texture_size.x, self.texture_size.y],
                front_color: self.front_color.into(),
            };
            if let Some(uniform) = &mut self.uniform {
                if data != **uniform {
                    **uniform = data;
                    uniform.sync(context);
                }
            } else {
                self.uniform = Some(Uniform::new(
                    data,
                    Self::BINDING,
                    &context.material_bind_group_layout,
                    &format!("material_{:?}", &self.key),
                    &context.device,
                ));
            }
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
                    .map_or(false, |t| t.inner().is_transparent);
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
    pub(crate) front_color: [f32; 4],
}
