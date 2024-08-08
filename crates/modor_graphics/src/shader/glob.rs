use crate::anti_aliasing::SupportedAntiAliasingModes;
use crate::gpu::{Gpu, GpuManager};
use crate::mesh::Vertex;
use crate::mesh::VertexBuffer;
use crate::model::Instance;
use crate::shader::loaded::ShaderLoaded;
use crate::{validation, AntiAliasingMode, Texture, Window};
use fxhash::FxHashMap;
use modor::{App, FromApp, Global};
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

// TODO: merge with mod.rs

/// The global data of a [`Shader`](crate::Shader).
#[derive(Debug, Global)]
pub struct ShaderGlobInner {
    pub(crate) material_bind_group_layout: BindGroupLayout,
    pub(crate) texture_count: u32,
    pub(crate) pipelines: FxHashMap<(TextureFormat, AntiAliasingMode), RenderPipeline>,
}

impl FromApp for ShaderGlobInner {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let loaded = ShaderLoaded::default();
        Self {
            material_bind_group_layout: Self::create_material_bind_group_layout(&gpu, &loaded),
            texture_count: 0,
            pipelines: FxHashMap::default(),
        }
    }
}

impl ShaderGlobInner {
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as VertexBuffer<0>>::LAYOUT,
        <Instance as VertexBuffer<
            { <Vertex as VertexBuffer<0>>::ATTRIBUTES.len() as u32 },
        >>::LAYOUT,
    ];

    pub(super) fn new(
        app: &mut App,
        loaded: &ShaderLoaded,
        is_alpha_replaced: bool,
        instance_size: usize,
    ) -> Result<Self, wgpu::Error> {
        let window_texture_format = app.get_mut::<Window>().texture_format();
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let material_bind_group_layout = Self::create_material_bind_group_layout(&gpu, loaded);
        Ok(Self {
            texture_count: loaded.texture_count,
            pipelines: [window_texture_format, Some(Texture::DEFAULT_FORMAT)]
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
                        Self::create_pipeline(
                            &gpu,
                            loaded,
                            format,
                            anti_aliasing,
                            is_alpha_replaced,
                            instance_size,
                            &material_bind_group_layout,
                        )?,
                    ))
                })
                .collect::<Result<FxHashMap<_, _>, _>>()?,
            material_bind_group_layout,
        })
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
        gpu: &Gpu,
        loaded: &ShaderLoaded,
        texture_format: TextureFormat,
        anti_aliasing: AntiAliasingMode,
        is_alpha_replaced: bool,
        instance_size: usize,
        material_bind_group_layout: &BindGroupLayout,
    ) -> Result<RenderPipeline, wgpu::Error> {
        validation::validate_wgpu(gpu, false, || {
            let module = gpu.device.create_shader_module(ShaderModuleDescriptor {
                label: Some("modor_shader"),
                source: wgpu::ShaderSource::Wgsl(loaded.code.as_str().into()),
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
            if instance_size > 0 {
                buffer_layout.push(VertexBufferLayout {
                    array_stride: instance_size as BufferAddress,
                    step_mode: VertexStepMode::Instance,
                    attributes: &loaded.instance_vertex_attributes,
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
