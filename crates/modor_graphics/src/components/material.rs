use crate::components::renderer::GpuContext;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::gpu_data::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::{Color, Renderer, Texture, TextureAnimation, DEFAULT_SHADER};
use modor::{Custom, SingleRef};
use modor_math::Vec2;
use modor_resources::{ResKey, Resource, ResourceAccessor, ResourceRegistry, ResourceState};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Sampler, TextureView};

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
/// # Examples
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
#[must_use]
#[derive(Component, Debug)]
pub struct Material {
    /// Color of the rendered instance.
    ///
    /// This color is multiplied to the texture when a [`texture_key`](#structfield.texture_key)
    /// is defined.
    ///
    /// Default is [`Color::WHITE`].
    pub color: Color,
    /// Key of the [`Texture`] used to render the instance.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
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
    /// aspect ratio is always kept during rendering. For example with a rectangle instance:
    /// - Main texture is stretched to cover the whole rectangle, so the aspect ratio might not be
    /// kept.
    /// - Foreground texture is centered on the rectangle and keeps its aspect ratio,
    /// which means the texture might not cover the whole rectangle.
    ///
    /// For example, the foreground texture is useful for rendering a text that should not be
    /// stretched.
    ///
    /// If the texture is not loaded, then the instances attached to the material are not rendered.
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
    pub(crate) bind_group: Option<BindGroup>,
    key: ResKey<Self>,
    buffer: Option<DynamicBuffer<MaterialData>>,
    renderer_version: Option<u8>,
    old_texture_key: Option<ResKey<Texture>>,
    old_front_texture_key: Option<ResKey<Texture>>,
    old_shader_key: ResKey<Shader>,
}

#[systems]
impl Material {
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
            bind_group: None,
            buffer: None,
            renderer_version: None,
            old_texture_key: None,
            old_front_texture_key: None,
            old_shader_key: DEFAULT_SHADER,
        }
    }

    #[run_after(
        component(Renderer),
        component(TextureAnimation),
        component(Shader),
        component(ShaderRegistry),
        component(Texture),
        component(TextureRegistry)
    )]
    fn update(
        &mut self,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        shaders: Custom<ResourceAccessor<'_, Shader>>,
        textures: Custom<ResourceAccessor<'_, Texture>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.bind_group = None;
            self.buffer = None;
        }
        if let Some(context) = state.context() {
            if let (Some(shader), Some(texture), Some(front_texture)) = (
                shaders.get(self.shader_key),
                textures.get(self.texture_key.unwrap_or(WHITE_TEXTURE)),
                textures.get(self.front_texture_key.unwrap_or(INVISIBLE_TEXTURE)),
            ) {
                self.update_buffer(context);
                if self.bind_group.is_none()
                    || self.texture_key != self.old_texture_key
                    || self.front_texture_key != self.old_front_texture_key
                    || self.shader_key != self.old_shader_key
                    || shader.is_material_bind_group_layout_reloaded
                    || texture.is_reloaded
                    || front_texture.is_reloaded
                {
                    self.update_bind_group(
                        shader,
                        &texture.inner().view,
                        &texture.inner().sampler,
                        &front_texture.inner().view,
                        &front_texture.inner().sampler,
                        context,
                    );
                }
                self.old_texture_key = self.texture_key;
                self.old_front_texture_key = self.front_texture_key;
                self.old_shader_key = self.shader_key;
            } else {
                self.bind_group = None;
                self.buffer = None;
            }
        }
    }

    #[run_after(component(TextureRegistry), component(Texture))]
    fn update_transparency(&mut self, textures: Custom<ResourceAccessor<'_, Texture>>) {
        self.is_transparent = (self.color.a > 0. && self.color.a < 1.)
            || Self::is_texture_transparent(self.texture_key, &textures)
            || Self::is_texture_transparent(self.front_texture_key, &textures);
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

    fn update_bind_group(
        &mut self,
        shader: &Shader,
        back_view: &TextureView,
        back_sampler: &Sampler,
        front_view: &TextureView,
        front_sampler: &Sampler,
        context: &GpuContext,
    ) {
        self.bind_group = Some(
            context.device.create_bind_group(&BindGroupDescriptor {
                layout: shader
                    .material_bind_group_layout
                    .as_ref()
                    .expect("internal error: material bind group not initialized"),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: self
                            .buffer
                            .as_ref()
                            .expect("internal error: material buffer not initialized")
                            .resource(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(back_view),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(back_sampler),
                    },
                    BindGroupEntry {
                        binding: 3,
                        resource: BindingResource::TextureView(front_view),
                    },
                    BindGroupEntry {
                        binding: 4,
                        resource: BindingResource::Sampler(front_sampler),
                    },
                ],
                label: Some(&format!("modor_bind_group_material_{}", self.key.label())),
            }),
        );
    }

    fn update_buffer(&mut self, context: &GpuContext) {
        let data = MaterialData {
            color: self.color.into(),
            texture_part_position: [self.texture_position.x, self.texture_position.y],
            texture_part_size: [self.texture_size.x, self.texture_size.y],
            front_color: self.front_color.into(),
        };
        if let Some(buffer) = &mut self.buffer {
            if data != buffer[0] {
                buffer[0] = data;
                buffer.sync(context);
            }
        } else {
            self.buffer = Some(DynamicBuffer::new(
                vec![data],
                DynamicBufferUsage::Uniform,
                format!("modor_uniform_buffer_material_{}", &self.key.label()),
                &context.device,
            ));
        }
    }
}

impl Resource for Material {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.buffer.is_some() && self.bind_group.is_some() {
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
