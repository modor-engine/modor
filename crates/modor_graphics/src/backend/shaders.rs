use crate::backend::renderer::{Renderer, DEPTH_TEXTUER_FORMAT};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    Face, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, StencilState, VertexBufferLayout, VertexState,
};

pub(crate) struct Shader {
    pipeline: RenderPipeline,
}

impl Shader {
    pub(crate) fn new(
        descriptor: ShaderModuleDescriptor,
        vertex_buffer_layout: &[VertexBufferLayout<'_>],
        label_suffix: &str,
        renderer: &Renderer,
    ) -> Self {
        let pipeline_layout = Self::create_pipeline_layout(label_suffix, renderer);
        let shader = renderer.device().create_shader_module(&descriptor);
        Self {
            pipeline: Self::create_pipeline(
                renderer,
                vertex_buffer_layout,
                &pipeline_layout,
                &shader,
            ),
        }
    }

    pub(super) fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    fn create_pipeline_layout(label_suffix: &str, renderer: &Renderer) -> PipelineLayout {
        renderer
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&format!("modor_pipeline_layout_{}", label_suffix)),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            })
    }

    fn create_pipeline(
        renderer: &Renderer,
        vertex_buffer_layout: &[VertexBufferLayout],
        pipeline_layout: &PipelineLayout,
        shader: &ShaderModule,
    ) -> RenderPipeline {
        renderer
            .device()
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("modor_render_pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: vertex_buffer_layout,
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[ColorTargetState {
                        format: renderer.surface_config().format,
                        blend: Some(BlendState::ALPHA_BLENDING),
                        write_mask: ColorWrites::ALL,
                    }],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(DepthStencilState {
                    format: DEPTH_TEXTUER_FORMAT,
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
    }
}