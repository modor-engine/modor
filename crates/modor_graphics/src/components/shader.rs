use crate::components::instance_group::Instance;
use crate::components::mesh::Vertex;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{errors, AntiAliasing, GpuContext, Renderer};
use fxhash::FxHashMap;
use modor::SingleRef;
use modor_resources::{
    Load, ResKey, Resource, ResourceHandler, ResourceLoadingError, ResourceRegistry,
    ResourceSource, ResourceState,
};
use std::collections::hash_map::Entry;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, StencilState, TextureFormat, VertexBufferLayout, VertexState,
};

pub(crate) type ShaderRegistry = ResourceRegistry<Shader>;

/// A shader that defines a rendering logic.
///
/// # Requirements
///
/// The shader is effective only if:
/// - graphics [`module`](crate::module()) is initialized
/// - the shader is linked to a [`Material`](crate::Material).
///
/// # Related components
///
/// - [`Material`](crate::Material)
///
/// # Built-in shaders
///
/// - [`DEFAULT_SHADER`](crate::DEFAULT_SHADER)
/// - [`ELLIPSE_SHADER`](crate::ELLIPSE_SHADER)
///
/// # Code
///
/// This component only supports code in [WGSL](https://www.w3.org/TR/WGSL/) format.
///
/// # Examples
///
/// Example of supported WGSL code:
/// ```wgsl
/// struct CameraUniform {
///     transform: mat4x4<f32>,
/// };
///
/// struct MaterialUniform {
///     color: vec4<f32>,
///     texture_part_position: vec2<f32>,
///     texture_part_size: vec2<f32>,
///     front_color: vec4<f32>,
/// }
///
/// struct Vertex {
///     @location(0)
///     position: vec3<f32>,
///     @location(1)
///     texture_position: vec2<f32>,
/// };
///
/// struct Instance {
///     @location(2)
///     transform_0: vec4<f32>,
///     @location(3)
///     transform_1: vec4<f32>,
///     @location(4)
///     transform_2: vec4<f32>,
///     @location(5)
///     transform_3: vec4<f32>,
/// };
///
/// struct Fragment {
///     @builtin(position)
///     position: vec4<f32>,
/// };
///
/// @group(0)
/// @binding(0)
/// var<uniform> camera: CameraUniform;
///
/// @group(1)
/// @binding(0)
/// var<uniform> material: MaterialUniform;
///
/// @group(2)
/// @binding(0)
/// var texture: texture_2d<f32>;
///
/// @group(2)
/// @binding(1)
/// var texture_sampler: sampler;
///
/// @group(3)
/// @binding(0)
/// var front_texture: texture_2d<f32>;
///
/// @group(3)
/// @binding(1)
/// var front_texture_sampler: sampler;
///
/// @vertex
/// fn vs_main(vertex: Vertex, instance: Instance) -> Fragment {
///     let transform = mat4x4<f32>(
///         instance.transform_0,
///         instance.transform_1,
///         instance.transform_2,
///         instance.transform_3,
///     );
///     return Fragment(camera.transform * transform * vec4<f32>(vertex.position, 1.));
/// }
///
/// @fragment
/// fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
///     // Just render the model in red.
///     return vec4(1., 0., 0., 1.);
/// }
/// ```
///
/// It is highly recommended to define the same structure and bindings, as they correspond to the
/// data sent to the shader.
///
/// Then the shader can be defined:
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// fn custom_instance() -> impl BuiltEntity {
///     let shader_key = ResKey::unique("custom");
///     instance_2d(WINDOW_CAMERA_2D, None)
///         .updated(|m: &mut Material| m.shader_key = shader_key)
///         // shader can also be defined outside this entity to be used by multiple instances
///         .component(Shader::from_path(shader_key, "path/to/shader"))
/// }
/// ```
#[derive(Component, Debug)]
pub struct Shader {
    texture_count: u8,
    key: ResKey<Self>,
    pipelines: FxHashMap<TextureFormat, RenderPipeline>,
    handler: ResourceHandler<LoadedShader, &'static str>,
    code: Option<String>,
    error: Option<ResourceLoadingError>,
    sample_count: u32,
    renderer_version: Option<u8>,
}

#[systems]
impl Shader {
    pub(crate) const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;
    pub(crate) const TEXTURE_GROUP: u32 = 2;
    pub(crate) const FRONT_TEXTURE_GROUP: u32 = 3;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as VertexBuffer<0>>::LAYOUT,
        <Instance as VertexBuffer<
            { <Vertex as VertexBuffer<0>>::ATTRIBUTES.len() as u32 },
        >>::LAYOUT,
    ];

    /// Creates a new shader identified by a unique `key` and created from code `source`.
    pub fn new(key: ResKey<Self>, source: ShaderSource) -> Self {
        Self {
            texture_count: 2,
            key,
            pipelines: FxHashMap::default(),
            handler: ResourceHandler::new(source.into()),
            code: None,
            error: None,
            sample_count: 1,
            renderer_version: None,
        }
    }

    /// Creates a new shader identified by a unique `key` and created with given `code`.
    ///
    /// This method is equivalent to [`Shader::new`] with [`ShaderSource::String`] source.
    pub fn from_string(key: ResKey<Self>, code: &'static str) -> Self {
        Self::new(key, ShaderSource::String(code))
    }

    /// Creates a new shader identified by a unique `key` and created with a given code file `path`.
    ///
    /// This method is equivalent to [`Shader::new`] with [`ShaderSource::Path`] source.
    pub fn from_path(key: ResKey<Self>, path: impl Into<String>) -> Self {
        Self::new(key, ShaderSource::Path(path.into()))
    }

    #[run_after(component(Renderer), component(AntiAliasing))]
    fn update(
        &mut self,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        anti_aliasing: Option<SingleRef<'_, '_, AntiAliasing>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.pipelines.clear();
        }
        if let Some(context) = state.context() {
            self.handler.update::<Self>(self.key);
            if let Some(shader) = self.handler.resource() {
                self.code = Some(shader.code);
                self.error = None;
                self.pipelines.clear();
            }
            let anti_aliasing = anti_aliasing.as_ref().map(SingleRef::get);
            let result = self.update_anti_aliasing(anti_aliasing, context);
            self.update_error(result);
            let result = self.update_texture_formats(context);
            self.update_error(result);
        }
    }

    pub(crate) fn pipeline(&self, texture_format: TextureFormat) -> &RenderPipeline {
        self.pipelines
            .get(&texture_format)
            .expect("internal error: render pipeline not loaded")
    }

    /// Sets the shader `source` and start reloading of the shader.
    ///
    /// If the previous source is already loaded, the shader remains valid until the new source
    /// is loaded.
    pub fn set_source(&mut self, source: ShaderSource) {
        self.handler.set_source(source.into());
    }

    fn update_error(&mut self, result: Result<(), wgpu::Error>) {
        if let Err(error) = result {
            self.error = Some(ResourceLoadingError::LoadingError(format!("{error}")));
        }
    }

    fn update_anti_aliasing(
        &mut self,
        anti_aliasing: Option<&AntiAliasing>,
        context: &GpuContext,
    ) -> Result<(), wgpu::Error> {
        let Some(code) = &self.code else {
            return Ok(());
        };
        let sample_count = anti_aliasing.map_or(1, |a| a.mode.sample_count());
        if self.sample_count != sample_count {
            self.sample_count = sample_count;
            for (texture_format, pipeline) in &mut self.pipelines {
                *pipeline = Self::create_pipeline(
                    self.texture_count,
                    code,
                    self.key,
                    *texture_format,
                    self.sample_count,
                    context,
                )?;
            }
        }
        Ok(())
    }

    fn update_texture_formats(&mut self, context: &GpuContext) -> Result<(), wgpu::Error> {
        let Some(code) = &self.code else {
            return Ok(());
        };
        let texture_formats = context.surface_texture_format.map_or_else(
            || vec![Self::TEXTURE_FORMAT],
            |format| vec![Self::TEXTURE_FORMAT, format],
        );
        for texture_format in texture_formats {
            if let Entry::Vacant(entry) = self.pipelines.entry(texture_format) {
                entry.insert(Self::create_pipeline(
                    self.texture_count,
                    code,
                    self.key,
                    texture_format,
                    self.sample_count,
                    context,
                )?);
            }
        }
        Ok(())
    }

    fn create_pipeline(
        texture_count: u8,
        code: &str,
        key: ResKey<Self>,
        texture_format: TextureFormat,
        sample_count: u32,
        context: &GpuContext,
    ) -> Result<RenderPipeline, wgpu::Error> {
        errors::validate_wgpu(context, || {
            let module = context.device.create_shader_module(ShaderModuleDescriptor {
                label: Some(&format!("modor_shader_{}", key.label())),
                source: wgpu::ShaderSource::Wgsl(code.into()),
            });
            let mut group_layouts = vec![
                &context.camera_bind_group_layout,
                &context.material_bind_group_layout,
            ];
            // TODO: put all textures in one bind group
            //     - Bind group layout is maintained in Shader
            //     - Bind group is maintained in Material
            //         - Material Uniform can also be put in the same binding group
            //     - Texture resources (view + sampler) are maintained in Texture
            for _ in 0..texture_count {
                group_layouts.push(&context.texture_bind_group_layout);
            }
            let layout = context
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&format!("modor_pipeline_layout_{}", key.label())),
                    bind_group_layouts: &group_layouts,
                    push_constant_ranges: &[],
                });
            context
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(&format!("modor_render_pipeline_{}", key.label())),
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs_main",
                        buffers: Self::VERTEX_BUFFER_LAYOUTS,
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs_main",
                        targets: &[Some(ColorTargetState {
                            format: texture_format,
                            blend: Some(BlendState::ALPHA_BLENDING),
                            write_mask: ColorWrites::ALL,
                        })],
                    }),
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: StencilState::default(),
                        bias: DepthBiasState::default(),
                    }),
                    multisample: MultisampleState {
                        count: sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                })
        })
    }
}

impl Resource for Shader {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if let Some(error) = &self.error {
            ResourceState::Error(error)
        } else if self.code.is_some() {
            ResourceState::Loaded
        } else {
            self.handler.state()
        }
    }
}

/// The code source of a [`Shader`].
///
/// Sources loaded synchronously are ready after the next [`App`](modor::App) update. Sources loaded
/// asynchronously can take more updates to be ready.
///
/// # Examples
///
/// See [`Shader`].
#[non_exhaustive]
#[derive(Debug)]
pub enum ShaderSource {
    /// Shader loaded synchronously from given code.
    ///
    /// This variant is generally used in combination with [`include_str!`].
    String(&'static str),
    /// Shader loaded asynchronously from a given path.
    ///
    /// # Platform-specific
    ///
    /// - Web: HTTP GET call is performed to retrieve the file from URL
    /// `{current_browser_url}/assets/{path}`.
    /// - Android: the file is retrieved using the Android
    /// [`AssetManager`](https://developer.android.com/reference/android/content/res/AssetManager).
    /// - Other: if `CARGO_MANIFEST_DIR` environment variable is set (this is the case if the
    /// application is run using a `cargo` command), then the file is retrieved from path
    /// `{CARGO_MANIFEST_DIR}/assets/{path}`. Else, the file path is
    /// `{executable_folder_path}/assets/{path}`.
    Path(String),
}

#[derive(Debug)]
struct LoadedShader {
    code: String,
}

impl Load<&'static str> for LoadedShader {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        String::from_utf8(data)
            .map(|code| Self { code })
            .map_err(|err| ResourceLoadingError::InvalidFormat(format!("{err}")))
    }

    fn load_from_data(data: &&'static str) -> Result<Self, ResourceLoadingError> {
        Ok(Self {
            code: (*data).to_string(),
        })
    }
}

impl From<ShaderSource> for ResourceSource<&'static str> {
    fn from(source: ShaderSource) -> Self {
        match source {
            ShaderSource::String(string) => ResourceSource::SyncData(string),
            ShaderSource::Path(path) => ResourceSource::AsyncPath(path),
        }
    }
}
