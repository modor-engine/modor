use crate::components::instance_group::Instance;
use crate::components::mesh::Vertex;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{errors, AntiAliasing, GpuContext, InstanceData, Renderer};
use fxhash::FxHashMap;
use modor::SingleRef;
use modor_resources::{
    Load, ResKey, Resource, ResourceHandler, ResourceLoadingError, ResourceRegistry,
    ResourceSource, ResourceState,
};
use regex::Regex;
use std::any::TypeId;
use std::collections::hash_map::Entry;
use std::str::FromStr;
use std::{any, mem};
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModuleDescriptor, ShaderStages,
    StencilState, TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
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
/// # Input locations
///
/// The code can include the following locations:
/// - location `0`: vertex position.
/// - location `1`: texture position for the vertex.
/// - location `2`: column 1 of the instance transform matrix.
/// - location `3`: column 2 of the instance transform matrix.
/// - location `4`: column 3 of the instance transform matrix.
/// - location `5`: column 4 of the instance transform matrix.
/// - location `6` or more: material data per instance. These locations must be defined
///     in a struct named `MaterialInstance` which corresponds to
///     [`MaterialSource::InstanceData`](crate::MaterialSource::InstanceData) on Rust side.
///
/// Below examples show how to define these locations correctly.
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
/// Below examples show how to define these bindings correctly.
///
/// # Examples
///
/// See [`MaterialSource`](crate::MaterialSource) for an example of supported WGSL code.
/// It is recommended to define the same structures and bindings.
#[derive(Component, Debug)]
pub struct Shader {
    pub(crate) material_bind_group_layout: Option<BindGroupLayout>,
    pub(crate) is_material_bind_group_layout_reloaded: bool,
    pub(crate) material_instance_type: TypeId,
    pub(crate) material_instance_type_name: &'static str,
    material_instance_size: usize,
    key: ResKey<Self>,
    pipelines: FxHashMap<(TextureFormat, bool), RenderPipeline>,
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
    ///
    /// `I` is the material data type per instance, it must be the same as
    /// [`MaterialSource::InstanceData`](crate::MaterialSource::InstanceData).
    pub fn new<I>(key: ResKey<Self>, source: ShaderSource) -> Self
    where
        I: InstanceData,
    {
        Self {
            material_bind_group_layout: None,
            is_material_bind_group_layout_reloaded: false,
            material_instance_type: TypeId::of::<I>(),
            material_instance_type_name: any::type_name::<I>(),
            material_instance_size: mem::size_of::<I>(),
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
    ///
    /// `I` is the material data type per instance, it must be the same as
    /// [`MaterialSource::InstanceData`](crate::MaterialSource::InstanceData).
    pub fn from_string<I>(key: ResKey<Self>, code: &'static str) -> Self
    where
        I: InstanceData,
    {
        Self::new::<I>(key, ShaderSource::String(code))
    }

    /// Creates a new shader identified by a unique `key` and created with a given code file `path`.
    ///
    /// This method is equivalent to [`Shader::new`] with [`ShaderSource::Path`] source.
    ///
    /// `I` is the material data type per instance, it should be the same as
    /// [`MaterialSource::InstanceData`](crate::MaterialSource::InstanceData).
    pub fn from_path<I>(key: ResKey<Self>, path: impl Into<String>) -> Self
    where
        I: InstanceData,
    {
        Self::new::<I>(key, ShaderSource::Path(path.into()))
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

    pub(crate) fn pipeline(
        &self,
        texture_format: TextureFormat,
        is_anti_aliasing_enabled: bool,
    ) -> &RenderPipeline {
        self.pipelines
            .get(&(texture_format, is_anti_aliasing_enabled))
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
            for (&(texture_format, is_anti_aliasing_enabled), pipeline) in &mut self.pipelines {
                if is_anti_aliasing_enabled {
                    *pipeline = Self::create_pipeline(
                        code,
                        self.key,
                        texture_format,
                        self.material_bind_group_layout
                            .as_ref()
                            .expect("internal error: material bind group not initialized"),
                        self.sample_count,
                        self.material_instance_size,
                        context,
                    )?;
                }
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
            for is_anti_aliasing_enabled in [false, true] {
                let key = (texture_format, is_anti_aliasing_enabled);
                let sample_count = if is_anti_aliasing_enabled {
                    self.sample_count
                } else {
                    1
                };
                if let Entry::Vacant(entry) = self.pipelines.entry(key) {
                    entry.insert(Self::create_pipeline(
                        code,
                        self.key,
                        texture_format,
                        self.material_bind_group_layout
                            .as_ref()
                            .expect("internal error: material bind group not initialized"),
                        sample_count,
                        self.material_instance_size,
                        context,
                    )?);
                }
            }
        }
        Ok(())
    }

    fn create_pipeline(
        code: &LoadedCode,
        key: ResKey<Self>,
        texture_format: TextureFormat,
        texture_bind_group_layout: &BindGroupLayout,
        sample_count: u32,
        material_instance_size: usize,
        context: &GpuContext,
    ) -> Result<RenderPipeline, wgpu::Error> {
        errors::validate_wgpu(context, || {
            let module = context.device.create_shader_module(ShaderModuleDescriptor {
                label: Some(&format!("modor_shader_{}", key.label())),
                source: wgpu::ShaderSource::Wgsl(code.string.as_str().into()),
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
            let mut buffer_layout = Self::VERTEX_BUFFER_LAYOUTS.to_vec();
            if material_instance_size > 0 {
                buffer_layout.push(VertexBufferLayout {
                    array_stride: material_instance_size as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &code.instance_vertex_attributes,
                });
            }
            context
                .device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(&format!("modor_render_pipeline_{}", key.label())),
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs_main",
                        buffers: &buffer_layout,
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

#[derive(Debug, PartialEq, Eq)]
struct LoadedCode {
    texture_count: usize,
    instance_vertex_attributes: Vec<VertexAttribute>,
    string: String,
}

impl LoadedCode {
    fn extract_texture_count(code: &str) -> usize {
        let binding_count = Regex::new(r"@group\(1\)\s*@binding\(([0-9]+)\)")
            .expect("internal error: invalid texture count regex")
            .captures_iter(code)
            .filter_map(|c| usize::from_str(&c[1]).ok())
            .max()
            .unwrap_or(0);
        (binding_count + 1).div_euclid(2)
    }

    fn extract_material_instance_struct(code: &str) -> Option<String> {
        Regex::new(r"(?s)struct\s+MaterialInstance\s*\{[^}]*}")
            .expect("internal error: invalid material instance struct regex")
            .captures(code)
            .map(|capture| capture[0].to_string())
    }

    fn extract_material_instance_layout(struct_code: &str) -> Result<Vec<VertexAttribute>, String> {
        let mut offset = 0;
        Regex::new(r"@location\(([0-9]+)\)\s*\w+\s*:\s*([\w<>]+)")
            .expect("internal error: invalid material instance layout regex")
            .captures_iter(struct_code)
            .filter_map(|capture| u32::from_str(&capture[1]).ok().map(|l| (capture, l)))
            .map(|(capture, shader_location)| {
                Self::format(&capture[2]).map(|format| {
                    let attribute = VertexAttribute {
                        format,
                        offset,
                        shader_location,
                    };
                    offset += format.size();
                    attribute
                })
            })
            .collect()
    }

    fn format(type_: &str) -> Result<VertexFormat, String> {
        match type_ {
            "f32" => Ok(VertexFormat::Float32),
            "i32" => Ok(VertexFormat::Sint32),
            "u32" => Ok(VertexFormat::Uint32),
            "vec2<f32>" => Ok(VertexFormat::Float32x2),
            "vec2<i32>" => Ok(VertexFormat::Sint32x2),
            "vec2<u32>" => Ok(VertexFormat::Uint32x2),
            "vec3<f32>" => Ok(VertexFormat::Float32x3),
            "vec3<i32>" => Ok(VertexFormat::Sint32x3),
            "vec3<u32>" => Ok(VertexFormat::Uint32x3),
            "vec4<f32>" => Ok(VertexFormat::Float32x4),
            "vec4<i32>" => Ok(VertexFormat::Sint32x4),
            "vec4<u32>" => Ok(VertexFormat::Uint32x4),
            _ => Err(format!(
                "WGSL type `{type_}` not supported in MaterialInstance"
            )),
        }
    }
}

impl TryFrom<String> for LoadedCode {
    type Error = String;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        let instance_struct = Self::extract_material_instance_struct(&string);
        Ok(Self {
            texture_count: Self::extract_texture_count(&string),
            instance_vertex_attributes: if let Some(instance_struct) = instance_struct {
                Self::extract_material_instance_layout(&instance_struct)
            } else {
                Ok(vec![])
            }?,
            string,
        })
    }
}

impl Load<&'static str> for LoadedCode {
    fn load_from_file(data: Vec<u8>) -> Result<Self, ResourceLoadingError> {
        let code = String::from_utf8(data)
            .map_err(|err| ResourceLoadingError::InvalidFormat(format!("{err}")))?;
        Self::try_from(code).map_err(ResourceLoadingError::InvalidFormat)
    }

    fn load_from_data(data: &&'static str) -> Result<Self, ResourceLoadingError> {
        Self::try_from((*data).to_string()).map_err(ResourceLoadingError::InvalidFormat)
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

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod loaded_code_tests {
    use crate::components::shader::LoadedCode;
    use wgpu::{VertexAttribute, VertexFormat};

    #[modor_test]
    fn load_code_with_no_material() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.string, code);
        assert_eq!(loaded_code.texture_count, 0);
    }

    #[modor_test]
    fn load_code_with_no_texture() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.string, code);
        assert_eq!(loaded_code.texture_count, 0);
    }

    #[modor_test]
    fn load_code_with_one_texture() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;

        @group(1)
        @binding(1)
        var texture: texture_2d<f32>;

        @group(1)
        @binding(2)
        var texture_sampler: sampler;
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.string, code);
        assert_eq!(loaded_code.texture_count, 1);
    }

    #[modor_test]
    fn load_code_with_many_textures() {
        let code = "
        @group(0)
        @binding(0)
        var<uniform> camera: Camera;

        @group(1)
        @binding(0)
        var<uniform> material: Material;

        @group(1)
        @binding(1)
        var texture: texture_2d<f32>;

        @group(1)
        @binding(2)
        var texture_sampler: sampler;

        @group(1)
        @binding(3)
        var texture2: texture_2d<f32>;

        @group(1)
        @binding(4)
        var texture_sampler2: sampler;
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.string, code);
        assert_eq!(loaded_code.texture_count, 2);
    }

    #[modor_test]
    fn load_code_without_material_instance_struct() {
        let code = "
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.instance_vertex_attributes, vec![]);
    }

    #[modor_test]
    fn load_code_with_empty_material_instance_struct() {
        let code = "
        struct MaterialInstance {
        }
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(loaded_code.instance_vertex_attributes, vec![]);
    }

    #[modor_test]
    fn load_code_with_one_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: u32
        }
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(
            loaded_code.instance_vertex_attributes,
            vec![VertexAttribute {
                format: VertexFormat::Uint32,
                offset: 0,
                shader_location: 6,
            }]
        );
    }

    #[modor_test]
    fn load_code_with_many_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: u32,
            @location(7)
            variable: vec2<u32>,
            @location(8)
            variable: vec3<u32>,
            @location(9)
            variable: vec4<u32>,
            @location(10)
            variable: i32,
            @location(11)
            variable: vec2<i32>,
            @location(12)
            variable: vec3<i32>,
            @location(13)
            variable: vec4<i32>,
            @location(14)
            variable: f32,
            @location(15)
            variable: vec2<f32>,
            @location(16)
            variable: vec3<f32>,
            @location(17)
            variable: vec4<f32>,
        }
        ";
        let loaded_code = LoadedCode::try_from(code.to_string()).unwrap();
        assert_eq!(
            loaded_code.instance_vertex_attributes,
            vec![
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: 0,
                    shader_location: 6,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x2,
                    offset: 4,
                    shader_location: 7,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x3,
                    offset: 12,
                    shader_location: 8,
                },
                VertexAttribute {
                    format: VertexFormat::Uint32x4,
                    offset: 24,
                    shader_location: 9,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32,
                    offset: 40,
                    shader_location: 10,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x2,
                    offset: 44,
                    shader_location: 11,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x3,
                    offset: 52,
                    shader_location: 12,
                },
                VertexAttribute {
                    format: VertexFormat::Sint32x4,
                    offset: 64,
                    shader_location: 13,
                },
                VertexAttribute {
                    format: VertexFormat::Float32,
                    offset: 80,
                    shader_location: 14,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: 84,
                    shader_location: 15,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 92,
                    shader_location: 16,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 104,
                    shader_location: 17,
                },
            ]
        );
    }

    #[modor_test]
    fn load_code_with_unsupported_material_instance_location() {
        let code = "
        struct MaterialInstance {
            @location(6)
            variable: i8
        }
        ";
        let loaded_code = LoadedCode::try_from(code.to_string());
        assert_eq!(
            loaded_code,
            Err("WGSL type `i8` not supported in MaterialInstance".to_string())
        );
    }
}
