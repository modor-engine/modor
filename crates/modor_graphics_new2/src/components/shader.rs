use crate::components::instances::Instance;
use crate::components::mesh::Vertex;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{
    IntoResourceKey, Renderer, RendererInner, Resource, ResourceKey, ResourceRegistry,
    ResourceState,
};
use modor::Single;
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, StencilState, TextureFormat, VertexBufferLayout,
    VertexState,
};

pub(crate) type ShaderRegistry = ResourceRegistry<Shader>;

#[derive(Component, Debug)]
pub(crate) struct Shader {
    code: String,
    key: ResourceKey,
    texture_format: TextureFormat,
    pipeline: Option<RenderPipeline>,
    renderer_version: Option<u8>,
}

impl Default for Shader {
    fn default() -> Self {
        Self::from_memory(
            ShaderKey::Default,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")),
        )
    }
}

#[systems]
impl Shader {
    pub(crate) const DEFAULT_TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;
    pub(crate) const CAMERA_GROUP: u32 = 0;
    pub(crate) const MATERIAL_GROUP: u32 = 1;
    pub(crate) const TEXTURE_GROUP: u32 = 2;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as VertexBuffer<0>>::LAYOUT,
        <Instance as VertexBuffer<
            { <Vertex as VertexBuffer<0>>::ATTRIBUTES.len() as u32 },
        >>::LAYOUT,
    ];

    pub(crate) fn ellipse() -> Self {
        Self::from_memory(
            ShaderKey::Ellipse,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
        )
    }

    fn from_memory(key: impl IntoResourceKey, code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            key: key.into_key(),
            texture_format: Self::DEFAULT_TEXTURE_FORMAT,
            pipeline: None,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer))]
    fn update(&mut self, renderer: Option<Single<'_, Renderer>>) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.pipeline = None;
        }
        if let Some(renderer) = state.renderer() {
            let texture_format = renderer
                .surface_texture_format
                .unwrap_or(self.texture_format);
            let pipeline = if texture_format == self.texture_format {
                self.pipeline.take()
            } else {
                self.texture_format = texture_format;
                None
            };
            self.pipeline = pipeline.or_else(|| {
                Some(Self::create_pipeline(
                    &self.code,
                    &self.key,
                    texture_format,
                    &renderer,
                ))
            })
        }
    }

    pub(crate) fn pipeline(&self) -> &RenderPipeline {
        self.pipeline
            .as_ref()
            .expect("internal error: render pipeline not loaded")
    }

    fn create_pipeline(
        code: &str,
        key: &ResourceKey,
        target_format: TextureFormat,
        renderer: &RendererInner,
    ) -> RenderPipeline {
        let module = renderer
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some(&format!("modor_shader_{:?}", key)),
                source: ShaderSource::Wgsl(code.into()),
            });
        let layout = renderer
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&format!("modor_pipeline_layout_{:?}", key)),
                bind_group_layouts: &[
                    &renderer.camera_bind_group_layout,
                    &renderer.material_bind_group_layout,
                    &renderer.texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        renderer
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&format!("modor_render_pipeline_{:?}", key)),
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
                        format: target_format,
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
                multisample: MultisampleState::default(),
                multiview: None,
            })
    }
}

impl Resource for Shader {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.pipeline.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShaderKey {
    Default,
    Ellipse,
}
