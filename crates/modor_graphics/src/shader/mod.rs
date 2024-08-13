use crate::anti_aliasing::SupportedAntiAliasingModes;
use crate::gpu::{Gpu, GpuManager};
use crate::mesh::{Vertex, VertexBuffer};
use crate::model::Instance;
use crate::shader::loaded::ShaderLoaded;
use crate::{validation, AntiAliasingMode, Material, Texture, Window};
use derivative::Derivative;
use fxhash::FxHashMap;
use getset::CopyGetters;
use log::error;
use modor::{App, FromApp, Glob, GlobRef, Update, Updater};
use modor_resources::{Res, ResSource, ResUpdater, Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;
use std::sync::Arc;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModuleDescriptor, ShaderStages,
    StencilState, TextureFormat, TextureSampleType, TextureViewDimension, VertexBufferLayout,
    VertexState, VertexStepMode,
};

mod loaded;

/// A [`Shader`] glob.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct ShaderGlob<T: Material> {
    inner: Glob<Res<Shader>>,
    phantom: PhantomData<fn(T)>,
}

impl<T> FromApp for ShaderGlob<T>
where
    T: Material,
{
    fn from_app(app: &mut App) -> Self {
        Self {
            inner: Glob::<Res<Shader>>::from_app_with(app, |res, app| {
                res.get_mut(app).instance_size = mem::size_of::<T::InstanceData>();
            }),
            phantom: PhantomData,
        }
    }
}

impl<T> Deref for ShaderGlob<T>
where
    T: Material,
{
    type Target = Glob<Res<Shader>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> ShaderGlob<T>
where
    T: Material,
{
    /// Returns static reference to the glob.
    pub fn to_ref(&self) -> ShaderGlobRef<T> {
        ShaderGlobRef {
            inner: self.inner.to_ref(),
            phantom: PhantomData,
        }
    }
}

/// A [`Shader`] glob reference.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = ""),
    Clone(bound = "")
)]
pub struct ShaderGlobRef<T> {
    inner: GlobRef<Res<Shader>>,
    phantom: PhantomData<T>,
}

impl<T> Deref for ShaderGlobRef<T>
where
    T: Material,
{
    type Target = GlobRef<Res<Shader>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A shader that defines a rendering logic.
///
/// # Supported languages
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
///     [`Material::InstanceData`] on Rust side.
///
/// # Bindings
///
/// The code can include the following bindings:
/// - group `0`
///     - binding `0`: camera data
/// - group `1`
///     - binding `0`: material data (`Material` struct corresponds to
///         [`Material::Data`] on Rust side)
///     - binding `(i * 2)`: `texture_2d<f32>` value corresponding to texture `i`
///     - binding `(i * 2 + 1)`: `sampler` value corresponding to texture `i`
///
/// # Examples
///
/// See [`Material`].
#[derive(Debug, Updater, CopyGetters)]
pub struct Shader {
    /// Controls how alpha channel should be treated:
    /// - `false`: apply standard alpha blending with non-premultiplied alpha.
    ///     It means models rendered behind a transparent model might be visible.
    /// - `true`: don't apply any color blending, just overwrites the output color.
    ///     It means models rendered behind a transparent model will never be visible.
    ///
    /// Default is `false`.
    #[getset(get_copy = "pub")]
    #[updater(field, for_field)]
    is_alpha_replaced: bool,
    /// General resource parameters.
    #[updater(inner_type, field)]
    res: PhantomData<ResUpdater<Shader>>,
    pub(crate) material_bind_group_layout: BindGroupLayout,
    pub(crate) pipelines: FxHashMap<(TextureFormat, AntiAliasingMode), RenderPipeline>,
    pub(crate) texture_count: u32,
    instance_size: usize,
    source: ResSource<Self>,
    loaded: ShaderLoaded,
    is_invalid: bool,
}

impl FromApp for Shader {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let loaded = ShaderLoaded::default();
        Self {
            is_alpha_replaced: false,
            res: PhantomData,
            material_bind_group_layout: Self::create_material_bind_group_layout(&gpu, &loaded),
            pipelines: FxHashMap::default(),
            texture_count: loaded.texture_count,
            instance_size: 0,
            source: ResSource::from_app(app),
            loaded,
            is_invalid: false,
        }
    }
}

impl Resource for Shader {
    type Source = ShaderSource;
    type Loaded = ShaderLoaded;

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        let code =
            String::from_utf8(file_bytes).map_err(|err| ResourceError::Other(format!("{err}")))?;
        ShaderLoaded::new(code)
    }

    fn load_from_source(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        ShaderLoaded::new(match source {
            ShaderSource::String(string) => string.clone(),
        })
    }

    fn on_load(&mut self, app: &mut App, loaded: Self::Loaded, source: &ResSource<Self>) {
        self.loaded = loaded;
        self.source = source.clone();
        self.update(app);
    }
}

impl Shader {
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as VertexBuffer<0>>::LAYOUT,
        <Instance as VertexBuffer<
            { <Vertex as VertexBuffer<0>>::ATTRIBUTES.len() as u32 },
        >>::LAYOUT,
    ];

    /// Whether an error occurred during parsing of the shader code.
    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn update(&mut self, app: &mut App) {
        let window_texture_format = app.get_mut::<Window>().texture_format();
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let material_bind_group_layout =
            Self::create_material_bind_group_layout(&gpu, &self.loaded);
        let pipelines = [window_texture_format, Some(Texture::DEFAULT_FORMAT)]
            .into_iter()
            .flatten()
            .flat_map(|format| {
                app.get_mut::<SupportedAntiAliasingModes>()
                    .get(&gpu, format)
                    .iter()
                    .copied()
                    .map(move |anti_aliasing| (format, anti_aliasing))
                    .collect::<Vec<_>>()
            })
            .map(|(format, anti_aliasing)| {
                Ok((
                    (format, anti_aliasing),
                    self.create_pipeline(&gpu, format, anti_aliasing, &material_bind_group_layout)?,
                ))
            })
            .collect::<Result<FxHashMap<_, _>, wgpu::Error>>();
        self.is_invalid = pipelines.is_err();
        match pipelines {
            Ok(pipelines) => {
                self.material_bind_group_layout = material_bind_group_layout;
                self.pipelines = pipelines;
                self.texture_count = self.loaded.texture_count;
            }
            Err(err) => {
                error!(
                    "Loading of shader from `{:?}` has failed: {err}",
                    self.source
                );
            }
        }
    }

    fn create_material_bind_group_layout(gpu: &Arc<Gpu>, loaded: &ShaderLoaded) -> BindGroupLayout {
        gpu.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &Self::create_bind_group_layout_entries(loaded),
                label: Some("modor_bind_group_layout_texture"),
            })
    }

    fn create_bind_group_layout_entries(loaded: &ShaderLoaded) -> Vec<BindGroupLayoutEntry> {
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
        for i in 0..loaded.texture_count {
            entries.extend([
                BindGroupLayoutEntry {
                    binding: i * 2 + 1,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: i * 2 + 2,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ]);
        }
        entries
    }

    fn create_pipeline(
        &self,
        gpu: &Gpu,
        texture_format: TextureFormat,
        anti_aliasing: AntiAliasingMode,
        material_bind_group_layout: &BindGroupLayout,
    ) -> Result<RenderPipeline, wgpu::Error> {
        validation::validate_wgpu(gpu, false, || {
            let module = gpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("modor_shader"),
                source: wgpu::ShaderSource::Wgsl(self.loaded.code.as_str().into()),
            });
            let layout = gpu
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("modor_pipeline_layout"),
                    bind_group_layouts: &[
                        &gpu.camera_bind_group_layout,
                        material_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let mut buffer_layout = Self::VERTEX_BUFFER_LAYOUTS.to_vec();
            if self.instance_size > 0 {
                buffer_layout.push(VertexBufferLayout {
                    array_stride: self.instance_size as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &self.loaded.instance_vertex_attributes,
                });
            }
            gpu.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some("modor_render_pipeline"),
                    layout: Some(&layout),
                    vertex: VertexState {
                        module: &module,
                        entry_point: "vs_main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        buffers: &buffer_layout,
                    },
                    fragment: Some(FragmentState {
                        module: &module,
                        entry_point: "fs_main",
                        compilation_options: wgpu::PipelineCompilationOptions::default(),
                        targets: &[Some(ColorTargetState {
                            format: texture_format,
                            blend: Some(if self.is_alpha_replaced {
                                BlendState::REPLACE
                            } else {
                                BlendState::ALPHA_BLENDING
                            }),
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
                        count: anti_aliasing.sample_count(),
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                    cache: None,
                })
        })
    }
}

impl ShaderUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &Glob<Res<Shader>>) {
        glob.take(app, |shader, app| {
            if Update::apply_checked(&mut self.is_alpha_replaced, &mut shader.is_alpha_replaced) {
                shader.update(app);
            }
        });
        if let Some(res) = self.res.take_value(|| unreachable!()) {
            res.apply(app, glob);
        }
    }
}

/// The source of a [`Shader`].
///
/// # Examples
///
/// See [`Shader`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ShaderSource {
    /// Shader code as a string.
    String(String),
}

impl Default for ShaderSource {
    fn default() -> Self {
        Self::String(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/empty.wgsl")).into())
    }
}

impl Source for ShaderSource {
    fn is_async(&self) -> bool {
        false
    }
}
