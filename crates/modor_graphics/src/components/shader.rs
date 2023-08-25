use crate::components::instances::Instance;
use crate::components::mesh::Vertex;
use crate::gpu_data::vertex_buffer::VertexBuffer;
use crate::{AntiAliasing, GpuContext, Renderer};
use fxhash::FxHashMap;
use modor::SingleRef;
use modor_resources::{ResKey, Resource, ResourceRegistry, ResourceState};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState,
    FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PolygonMode,
    PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, StencilState, TextureFormat, VertexBufferLayout,
    VertexState,
};

pub(crate) type ShaderRegistry = ResourceRegistry<Shader>;

pub(crate) const DEFAULT_SHADER: ResKey<Shader> = ResKey::new("default(modor_graphics)");
pub(crate) const ELLIPSE_SHADER: ResKey<Shader> = ResKey::new("ellipse(modor_graphics)");

#[derive(Component, Debug)]
pub(crate) struct Shader {
    code: String,
    key: ResKey<Self>,
    pipelines: FxHashMap<TextureFormat, RenderPipeline>,
    sample_count: u32,
    renderer_version: Option<u8>,
}

impl Default for Shader {
    fn default() -> Self {
        Self::from_memory(
            DEFAULT_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")),
        )
    }
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

    pub(crate) fn ellipse() -> Self {
        Self::from_memory(
            ELLIPSE_SHADER,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
        )
    }

    fn from_memory(key: ResKey<Self>, code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            key,
            pipelines: FxHashMap::default(),
            sample_count: 1,
            renderer_version: None,
        }
    }

    #[run_after(component(Renderer))]
    fn update(
        &mut self,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        anti_aliasing: Option<SingleRef<'_, '_, AntiAliasing>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.pipelines.clear();
        }
        let anti_aliasing = anti_aliasing.as_ref().map(SingleRef::get).copied();
        state.context().map_or_else(
            || unreachable!("internal error: unreachable shader state"),
            |context| {
                self.update_anti_aliasing(anti_aliasing, context);
                self.update_texture_formats(context);
            },
        );
    }

    fn update_anti_aliasing(&mut self, anti_aliasing: Option<AntiAliasing>, context: &GpuContext) {
        let sample_count = anti_aliasing.map_or(1, AntiAliasing::sample_count);
        if self.sample_count != sample_count {
            self.sample_count = sample_count;
            for (texture_format, pipeline) in &mut self.pipelines {
                *pipeline = Self::create_pipeline(
                    &self.code,
                    self.key,
                    *texture_format,
                    self.sample_count,
                    context,
                );
            }
        }
    }

    fn update_texture_formats(&mut self, context: &GpuContext) {
        let texture_formats = context.surface_texture_format.map_or_else(
            || vec![Self::TEXTURE_FORMAT],
            |format| vec![Self::TEXTURE_FORMAT, format],
        );
        for texture_format in texture_formats {
            self.pipelines.entry(texture_format).or_insert_with(|| {
                Self::create_pipeline(
                    &self.code,
                    self.key,
                    texture_format,
                    self.sample_count,
                    context,
                )
            });
        }
    }

    pub(crate) fn pipeline(&self, texture_format: TextureFormat) -> &RenderPipeline {
        self.pipelines
            .get(&texture_format)
            .expect("internal error: render pipeline not loaded")
    }

    fn create_pipeline(
        code: &str,
        key: ResKey<Self>,
        texture_format: TextureFormat,
        sample_count: u32,
        context: &GpuContext,
    ) -> RenderPipeline {
        let module = context.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("modor_shader_{}", key.label())),
            source: ShaderSource::Wgsl(code.into()),
        });
        let layout = context
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(&format!("modor_pipeline_layout_{}", key.label())),
                bind_group_layouts: &[
                    &context.camera_bind_group_layout,
                    &context.material_bind_group_layout,
                    &context.texture_bind_group_layout,
                    &context.texture_bind_group_layout,
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
    }
}

impl Resource for Shader {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.pipelines.is_empty() {
            ResourceState::NotLoaded
        } else {
            ResourceState::Loaded
        }
    }
}
