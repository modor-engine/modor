use crate::gpu::{Gpu, GpuManager};
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

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct Shader<T> {
    pub is_alpha_replaced: bool,
    loaded: ShaderLoaded,
    glob: Glob<ShaderGlob>,
    is_invalid: bool,
    old_is_alpha_replaced: bool,
    phantom_data: PhantomData<T>,
}

impl<T> Resource for Shader<T>
where
    T: 'static + Material,
{
    type Source = ShaderSource;
    type Loaded = ShaderLoaded;

    fn create(ctx: &mut Context<'_>) -> Self {
        let loaded = ShaderLoaded::default();
        let glob = ShaderGlob::new::<T>(
            ctx,
            &loaded,
            Self::DEFAULT_IS_ALPHA_REPLACED,
            "empty(modor_graphics)",
        )
        .expect("internal error: cannot load empty shader");
        Self {
            is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
            glob: Glob::new(ctx, glob),
            loaded,
            is_invalid: false,
            old_is_alpha_replaced: Self::DEFAULT_IS_ALPHA_REPLACED,
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
        if let Some(loaded) = loaded {
            self.loaded = loaded;
            self.update(ctx, label);
        } else if self.is_alpha_replaced != self.old_is_alpha_replaced {
            self.update(ctx, label);
        }
    }
}

impl<T> Shader<T>
where
    T: 'static + Material,
{
    const DEFAULT_IS_ALPHA_REPLACED: bool = false;

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<ShaderGlob> {
        self.glob.as_ref()
    }

    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn update(&mut self, ctx: &mut Context<'_>, label: &str) {
        match ShaderGlob::new::<T>(ctx, &self.loaded, self.is_alpha_replaced, label) {
            Ok(glob) => {
                *self.glob.get_mut(ctx) = glob;
                self.is_invalid = false;
            }
            Err(err) => {
                self.is_invalid = true;
                error!("Loading of shader '{label}' has failed: {err}");
            }
        }
        self.old_is_alpha_replaced = self.is_alpha_replaced;
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
    pub(crate) texture_count: u32,
    pub(crate) pipelines: FxHashMap<TextureFormat, RenderPipeline>,
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

    fn new<T>(
        ctx: &mut Context<'_>,
        loaded: &ShaderLoaded,
        is_alpha_replaced: bool,
        label: &str,
    ) -> Result<Self, wgpu::Error>
    where
        T: 'static + Material,
    {
        let texture_formats = Self::texture_formats(ctx);
        let gpu = ctx.root::<GpuManager>().get_mut(ctx).get();
        let material_bind_group_layout = Self::material_bind_group_layout(gpu, loaded, label);
        Ok(Self {
            texture_count: loaded.texture_count,
            pipelines: texture_formats
                .into_iter()
                .map(|texture_format| {
                    Ok((
                        texture_format,
                        Self::pipeline::<T>(
                            gpu,
                            loaded,
                            texture_format,
                            is_alpha_replaced,
                            &material_bind_group_layout,
                            label,
                        )?,
                    ))
                })
                .collect::<Result<FxHashMap<_, _>, _>>()?,
            material_bind_group_layout,
        })
    }

    pub fn texture_formats(ctx: &mut Context<'_>) -> Vec<TextureFormat> {
        let mut formats = vec![Texture::DEFAULT_FORMAT];
        formats.extend(ctx.root::<Window>().get(ctx).texture_format());
        formats
    }

    fn material_bind_group_layout(
        gpu: &Arc<Gpu>,
        loaded: &ShaderLoaded,
        label: &str,
    ) -> BindGroupLayout {
        gpu.device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &Self::bind_group_layout_entries(loaded),
                label: Some(&format!("modor_bind_group_layout_texture:{label}")),
            })
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

    fn pipeline<T>(
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
