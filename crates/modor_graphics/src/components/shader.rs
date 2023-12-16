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
use regex::Regex;
use std::collections::hash_map::Entry;
use std::str::FromStr;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferBindingType, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModuleDescriptor, ShaderStages, StencilState, TextureFormat,
    TextureSampleType, TextureViewDimension, VertexBufferLayout, VertexState,
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
/// # Code
///
/// This component only supports code in [WGSL](https://www.w3.org/TR/WGSL/) format.
///
/// # Bindings
///
/// The code can include the following bindings:
/// - group `0`
///     - binding `0`: camera data as defined in the below example
/// - group `1`
///     - binding `0`: material data as defined in the below example
///         (fields can vary depending on the associated [`Material`](crate::Material)s)
///     - binding `(i * 2)`: `texture_2d<f32>` value corresponding to texture `i`
///     - binding `(i * 2 + 1)`: `sampler` value corresponding to texture `i`
///
/// The number of defined textures must be the same as the number of textures defined in the
/// associated [`Material`](crate::Material)s.
///
/// # Examples
///
/// Example of supported WGSL code:
/// ```wgsl
/// struct Camera {
///     transform: mat4x4<f32>,
/// };
///
/// struct Material {
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
/// var<uniform> camera: Camera;
///
/// @group(1)
/// @binding(0)
/// var<uniform> material: Material;
///
/// @group(1)
/// @binding(1)
/// var texture: texture_2d<f32>;
///
/// @group(1)
/// @binding(2)
/// var texture_sampler: sampler;
///
/// @group(1)
/// @binding(3)
/// var front_texture: texture_2d<f32>;
///
/// @group(1)
/// @binding(4)
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
/// It is recommended to define the same structures and bindings.
///
/// Then the shader can be defined and linked to a [`Material`](crate::Material).
#[derive(Component, Debug)]
pub struct Shader {
    pub(crate) material_bind_group_layout: Option<BindGroupLayout>,
    pub(crate) is_material_bind_group_layout_reloaded: bool,
    key: ResKey<Self>,
    pipelines: FxHashMap<TextureFormat, RenderPipeline>,
    handler: ResourceHandler<LoadedCode, &'static str>,
    code: Option<LoadedCode>,
    error: Option<ResourceLoadingError>,
    sample_count: u32,
    renderer_version: Option<u8>,
}

#[systems]
impl Shader {
    pub(crate) const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;

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
            material_bind_group_layout: None,
            is_material_bind_group_layout_reloaded: false,
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
        self.is_material_bind_group_layout_reloaded = false;
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.pipelines.clear();
            self.material_bind_group_layout = None;
        }
        if let Some(context) = state.context() {
            self.handler.update::<Self>(self.key);
            if let Some(shader) = self.handler.resource() {
                self.code = Some(shader);
                self.error = None;
                self.pipelines.clear();
            }
            self.update_texture_bind_group(context);
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

    #[allow(clippy::cast_possible_truncation)]
    fn update_texture_bind_group(&mut self, context: &GpuContext) {
        if let (Some(code), None) = (&self.code, &self.material_bind_group_layout) {
            let mut entries = vec![BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }];
            for i in 0..code.texture_count {
                entries.extend([
                    BindGroupLayoutEntry {
                        binding: (i * 2 + 1) as u32,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: (i * 2 + 2) as u32,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ]);
            }
            self.material_bind_group_layout = Some(context.device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    entries: &entries,
                    label: Some(&format!(
                        "modor_bind_group_layout_texture_{}",
                        self.key.label(),
                    )),
                },
            ));
            self.is_material_bind_group_layout_reloaded = true;
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
                    &code.string,
                    self.key,
                    *texture_format,
                    self.material_bind_group_layout
                        .as_ref()
                        .expect("internal error: material bind group not initialized"),
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
                    &code.string,
                    self.key,
                    texture_format,
                    self.material_bind_group_layout
                        .as_ref()
                        .expect("internal error: material bind group not initialized"),
                    self.sample_count,
                    context,
                )?);
            }
        }
        Ok(())
    }

    fn create_pipeline(
        code: &str,
        key: ResKey<Self>,
        texture_format: TextureFormat,
        texture_bind_group_layout: &BindGroupLayout,
        sample_count: u32,
        context: &GpuContext,
    ) -> Result<RenderPipeline, wgpu::Error> {
        errors::validate_wgpu(context, || {
            let module = context.device.create_shader_module(ShaderModuleDescriptor {
                label: Some(&format!("modor_shader_{}", key.label())),
                source: wgpu::ShaderSource::Wgsl(code.into()),
            });
            let layout = context
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&format!("modor_pipeline_layout_{}", key.label())),
                    bind_group_layouts: &[
                        &context.camera_bind_group_layout,
                        texture_bind_group_layout,
                    ],
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
struct LoadedCode {
    texture_count: usize,
    string: String,
}

impl LoadedCode {
    fn extract_texture_count(code: &str) -> usize {
        let texture_binding_regex = Regex::new(r"@group\(1\)\s*@binding\(([0-9]+)\)")
            .expect("internal error: invalid texture count regex");
        let binding_count = texture_binding_regex
            .captures_iter(code)
            .filter_map(|c| usize::from_str(&c[1]).ok())
            .max()
            .unwrap_or(0);
        (binding_count + 1).div_euclid(2)
    }
}

impl Load<&'static str> for LoadedCode {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        String::from_utf8(data)
            .map(|string| Self {
                texture_count: Self::extract_texture_count(&string),
                string,
            })
            .map_err(|err| ResourceLoadingError::InvalidFormat(format!("{err}")))
    }

    fn load_from_data(data: &&'static str) -> Result<Self, ResourceLoadingError> {
        let string = (*data).to_string();
        Ok(Self {
            texture_count: Self::extract_texture_count(&string),
            string,
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
