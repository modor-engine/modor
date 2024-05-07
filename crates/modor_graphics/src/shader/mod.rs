use crate::gpu::{Gpu, GpuHandle, GpuResourceAction};
use crate::mesh::Vertex;
use crate::model::Instance;
use crate::shader::loaded::ShaderLoaded;
use crate::vertex_buffer::VertexBuffer;
use crate::{validation, Material, Texture, Window};
use derivative::Derivative;
use fxhash::FxHashMap;
use log::error;
use modor::{Context, Glob, GlobRef};
use modor_resources::{Resource, ResourceError, Source};
use std::marker::PhantomData;
use std::mem;
use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
    BufferAddress, BufferBindingType, ColorTargetState, ColorWrites, CompareFunction,
    DepthBiasState, DepthStencilState, FragmentState, FrontFace, MultisampleState,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, ShaderModuleDescriptor, ShaderStages,
    StencilState, TextureFormat, TextureSampleType, TextureViewDimension, VertexBufferLayout,
    VertexState, VertexStepMode,
};

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct Shader<T> {
    pub is_alpha_replaced: bool,
    loaded: Option<ShaderLoaded>,
    glob: Glob<Option<ShaderGlob>>,
    gpu: GpuHandle,
    is_invalid: bool,
    phantom_data: PhantomData<T>,
}

impl<T> Resource for Shader<T>
where
    T: 'static + Material,
{
    type Source = ShaderSource;
    type Loaded = ShaderLoaded;

    fn create(ctx: &mut Context<'_>) -> Self {
        Self {
            is_alpha_replaced: false,
            loaded: None,
            glob: Glob::new(ctx, None),
            gpu: GpuHandle::default(),
            is_invalid: false,
            phantom_data: PhantomData,
        }
    }

    fn load_from_file(file_bytes: Vec<u8>) -> Result<Self::Loaded, ResourceError> {
        let code =
            String::from_utf8(file_bytes).map_err(|err| ResourceError::Other(format!("{err}")))?;
        ShaderLoaded::new(code)
    }

    fn load(source: &Self::Source) -> Result<Self::Loaded, ResourceError> {
        ShaderLoaded::new(match source {
            ShaderSource::String(string) => string.clone(),
        })
    }

    fn update(&mut self, ctx: &mut Context<'_>, loaded: Option<Self::Loaded>, label: &str) {
        let is_loaded = loaded.is_some();
        if let Some(loaded) = loaded {
            self.loaded = Some(loaded);
        }
        match self.gpu.action(ctx, is_loaded) {
            GpuResourceAction::Delete => *self.glob.get_mut(ctx) = None,
            GpuResourceAction::Create(gpu) => self.create_glob(ctx, &gpu, label),
            GpuResourceAction::Update(gpu) => self.update_glob(ctx, &gpu, label),
        }
    }
}

impl<T> Shader<T>
where
    T: 'static + Material,
{
    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Option<ShaderGlob>> {
        self.glob.as_ref()
    }

    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn create_glob(&mut self, ctx: &mut Context<'_>, gpu: &Gpu, label: &str) {
        if let Some(loaded) = &self.loaded {
            *self.glob.get_mut(ctx) = Some(ShaderGlob::new(gpu, loaded, label));
            self.is_invalid = false;
        }
        self.update_glob(ctx, gpu, label);
    }

    fn update_glob(&mut self, ctx: &mut Context<'_>, gpu: &Gpu, label: &str) {
        if self.is_invalid {
            return;
        }
        let texture_formats = Self::texture_formats(ctx);
        if let (Some(glob), Some(loaded)) = (self.glob.get_mut(ctx), &self.loaded) {
            if glob
                .update::<T>(gpu, loaded, &texture_formats, self.is_alpha_replaced, label)
                .is_err()
            {
                self.is_invalid = true;
                error!("Loading of shader '{label}' has failed");
            }
        }
    }

    pub fn texture_formats(ctx: &mut Context<'_>) -> Vec<TextureFormat> {
        let mut formats = vec![Texture::DEFAULT_FORMAT];
        formats.extend(ctx.root::<Window>().get(ctx).texture_format());
        formats
    }
}

#[derive(Debug, Clone)]
pub enum ShaderSource {
    String(String),
}

impl Source for ShaderSource {
    fn is_async(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct ShaderGlob {
    pub(crate) material_bind_group_layout: BindGroupLayout,
    pipelines: FxHashMap<TextureFormat, RenderPipeline>,
    gpu_version: u64,
    is_alpha_replaced: bool,
}

impl ShaderGlob {
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as VertexBuffer<0>>::LAYOUT,
        <Instance as VertexBuffer<
            { <Vertex as VertexBuffer<0>>::ATTRIBUTES.len() as u32 },
        >>::LAYOUT,
    ];

    pub(crate) fn pipeline(
        &self,
        gpu_version: u64,
        texture_format: TextureFormat,
    ) -> Option<&RenderPipeline> {
        (gpu_version == self.gpu_version)
            .then(|| self.pipelines.get(&texture_format))
            .flatten()
    }

    fn new(gpu: &Gpu, loaded: &ShaderLoaded, label: &str) -> Self {
        Self {
            material_bind_group_layout: gpu.device.create_bind_group_layout(
                &BindGroupLayoutDescriptor {
                    entries: &Self::bind_group_layout_entries(loaded),
                    label: Some(&format!("modor_bind_group_layout_texture:{label}")),
                },
            ),
            pipelines: FxHashMap::default(),
            gpu_version: gpu.version,
            is_alpha_replaced: false,
        }
    }

    fn update<T>(
        &mut self,
        gpu: &Gpu,
        loaded: &ShaderLoaded,
        texture_formats: &[TextureFormat],
        is_alpha_replaced: bool,
        label: &str,
    ) -> Result<(), wgpu::Error>
    where
        T: 'static + Material,
    {
        let has_changed_property = self.is_alpha_replaced != is_alpha_replaced;
        for &texture_format in texture_formats {
            if has_changed_property || !self.pipelines.contains_key(&texture_format) {
                self.pipelines.insert(
                    texture_format,
                    Self::create_pipeline::<T>(
                        gpu,
                        loaded,
                        texture_format,
                        is_alpha_replaced,
                        &self.material_bind_group_layout,
                        label,
                    )?,
                );
            }
        }
        self.is_alpha_replaced = is_alpha_replaced;
        Ok(())
    }

    fn bind_group_layout_entries(loaded: &ShaderLoaded) -> Vec<BindGroupLayoutEntry> {
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

    fn create_pipeline<T>(
        gpu: &Gpu,
        loaded: &ShaderLoaded,
        texture_format: TextureFormat,
        is_alpha_replaced: bool,
        material_bind_group_layout: &BindGroupLayout,
        label: &str,
    ) -> Result<RenderPipeline, wgpu::Error>
    where
        T: 'static + Material,
    {
        validation::validate_wgpu(gpu, || {
            let module = gpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some(&format!("modor_shader:{label}")),
                source: wgpu::ShaderSource::Wgsl(loaded.code.as_str().into()),
            });
            let layout = gpu
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&format!("modor_pipeline_layout:{label}")),
                    bind_group_layouts: &[
                        &gpu.camera_bind_group_layout,
                        material_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });
            let mut buffer_layout = Self::VERTEX_BUFFER_LAYOUTS.to_vec();
            let instance_size = mem::size_of::<T::InstanceData>();
            if instance_size > 0 {
                buffer_layout.push(VertexBufferLayout {
                    array_stride: instance_size as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &loaded.instance_vertex_attributes,
                });
            }
            gpu.device
                .create_render_pipeline(&RenderPipelineDescriptor {
                    label: Some(&format!("modor_render_pipeline:{label}")),
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
                            blend: Some(if is_alpha_replaced {
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
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                })
        })
    }
}

mod loaded;
