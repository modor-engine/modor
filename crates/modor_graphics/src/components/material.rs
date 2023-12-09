use crate::components::shader::Shader;
use crate::components::texture::TextureRegistry;
use crate::gpu_data::uniform::Uniform;
use crate::{Color, Renderer, Texture, TextureAnimation, DEFAULT_SHADER};
use modor::{Custom, SingleRef};
use modor_math::Vec2;
use modor_resources::{ResKey, Resource, ResourceAccessor, ResourceRegistry, ResourceState};

pub(crate) type MaterialRegistry = ResourceRegistry<Material>;

/// The aspect of a rendered instance.
///
/// # Requirements
///
/// The material is effective only if:
/// - graphics [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`InstanceRendering2D`](crate::InstanceRendering2D)
/// - [`Texture`]
///
/// # Entity functions creating this component
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
///
/// # Performance
///
/// As models are rendered by batch based on the material, recreating the same material for each
/// model is less performant than creating it once for all models.
///
/// # Examples
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    /// Color of the rendered model.
    ///
    /// This color is multiplied to the texture when a [`texture_key`](#structfield.texture_key)
    /// is defined.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    /// Key of the [`Texture`] used to render the model.
    ///
    /// If the texture is not loaded, then the models attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub texture_key: Option<ResKey<Texture>>,
    /// Top-left position of the extracted texture section.
    ///
    /// [`Vec2::ZERO`] corresponds to top-left corner, and [`Vec2::ONE`] corresponds to bottom-right
    /// corner of the texture.
    ///
    /// Default is [`Vec2::ZERO`].
    pub texture_position: Vec2,
    /// Size of the extracted texture section.
    ///
    /// [`Vec2::ONE`] corresponds to the entire texture.
    ///
    /// Default is [`Vec2::ONE`].
    pub texture_size: Vec2,
    /// Key of the foreground texture.
    ///
    /// This texture is placed on top of the main texture defined using
    /// [`texture_key`](#structfield.texture_key). In contrary to the main texture, the initial
    /// aspect ratio is always kept during rendering. For example with a rectangle model:
    /// - Main texture is stretched to cover the whole rectangle, so the aspect ratio might not be
    /// kept.
    /// - Foreground texture is centered on the rectangle and keeps its aspect ratio,
    /// which means the texture might not cover the whole rectangle.
    ///
    /// For example, the foreground texture is useful for rendering a text that should not be
    /// stretched.
    ///
    /// If the texture is not loaded, then the models attached to the material are not rendered.
    ///
    /// Default is [`None`].
    pub front_texture_key: Option<ResKey<Texture>>,
    /// Color that is multiplied to the foreground texture when
    /// [`front_texture_key`](#structfield.front_texture_key) is defined.
    ///
    /// Default is [`Color::BLACK`].
    pub front_color: Color,
    /// Key of the [`Shader`].
    ///
    /// Default is [`DEFAULT_SHADER`].
    pub shader_key: ResKey<Shader>,
    pub(crate) is_transparent: bool,
    key: ResKey<Self>,
    uniform: Option<Uniform<MaterialData>>,
    renderer_version: Option<u8>,
}

#[systems]
impl Material {
    const BINDING: u32 = 0;

    /// Creates a new material with a unique `key`.
    pub fn new(key: ResKey<Self>) -> Self {
        Self {
            color: Color::WHITE,
            texture_key: None,
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            front_texture_key: None,
            front_color: Color::BLACK,
            shader_key: DEFAULT_SHADER,
            is_transparent: false,
            key,
            uniform: None,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer), component(TextureAnimation))]
    fn update_uniform(&mut self, renderer: Option<SingleRef<'_, '_, Renderer>>) {
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
                    &format!("material_{}", &self.key.label()),
                    &context.device,
                ));
            }
        }
    }

    #[run_after(component(TextureRegistry), component(Texture))]
    fn update_transparency(&mut self, textures: Custom<ResourceAccessor<'_, Texture>>) {
        self.is_transparent = (self.color.a > 0. && self.color.a < 1.)
            || Self::is_texture_transparent(self.texture_key, &textures)
            || Self::is_texture_transparent(self.front_texture_key, &textures);
    }

    pub(crate) fn uniform(&self) -> &Uniform<MaterialData> {
        self.uniform
            .as_ref()
            .expect("internal error: material uniform not initialized")
    }

    fn is_texture_transparent(
        texture_key: Option<ResKey<Texture>>,
        textures: &Custom<ResourceAccessor<'_, Texture>>,
    ) -> bool {
        texture_key
            .as_ref()
            .and_then(|&k| textures.get(k))
            .map_or(false, |t| t.inner().is_transparent)
    }
}

impl Resource for Material {
    fn key(&self) -> ResKey<Self> {
        self.key
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
