use crate::instances::Instance;
use crate::resources::buffers::GpuData;
use crate::resources::models::Vertex;
use crate::resources::registries::{Resource, ResourceRegistry};
use crate::ResourceKey;
use modor::{Built, EntityBuilder};
use modor_internal::dyn_types::DynType;
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, FrontFace, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, StencilState,
    TextureFormat, VertexBufferLayout, VertexState,
};

pub(crate) type ShaderRegistry = ResourceRegistry<Shader>;

pub(crate) struct Shader {
    key: DynType,
    pipeline: RenderPipeline,
}

#[component]
impl Shader {
    pub(crate) const CAMERA_GROUP: u32 = 0;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as GpuData<0>>::LAYOUT,
        <Instance as GpuData<{ <Vertex as GpuData<0>>::ATTRIBUTES.len() as u32 }>>::LAYOUT,
    ];

    fn new(
        key: impl ResourceKey,
        code: &str,
        target_format: TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> Self {
        let bind_group_layouts = &[camera_bind_group_layout];
        let pipeline_layout = Self::create_pipeline_layout(&key, bind_group_layouts, device);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("modor_shader_{:?}", key)),
            source: ShaderSource::Wgsl(code.into()),
        });
        Self {
            key: DynType::new(key),
            pipeline: Self::create_pipeline(device, &pipeline_layout, target_format, &shader),
        }
    }

    pub(crate) fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    fn create_pipeline_layout(
        key: &impl ResourceKey,
        bind_group_layouts: &[&BindGroupLayout],
        device: &Device,
    ) -> PipelineLayout {
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some(&format!("modor_pipeline_layout_{:?}", key)),
            bind_group_layouts,
            push_constant_ranges: &[],
        })
    }

    fn create_pipeline(
        device: &Device,
        pipeline_layout: &PipelineLayout,
        target_format: TextureFormat,
        shader: &ShaderModule,
    ) -> RenderPipeline {
        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("modor_render_pipeline"),
            layout: Some(pipeline_layout),
            vertex: VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: Self::VERTEX_BUFFER_LAYOUTS,
            },
            fragment: Some(FragmentState {
                module: shader,
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
    fn key(&self) -> &DynType {
        &self.key
    }
}

pub(crate) struct RectangleShader;

#[singleton]
impl RectangleShader {
    pub(crate) fn build(
        target_format: TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Shader::new(
            ShaderKey::Rectangle,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
            target_format,
            camera_bind_group_layout,
            device,
        ))
    }
}

pub(crate) struct EllipseShader;

#[singleton]
impl EllipseShader {
    pub(crate) fn build(
        target_format: TextureFormat,
        camera_bind_group_layout: &BindGroupLayout,
        device: &Device,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Shader::new(
            ShaderKey::Ellipse,
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
            target_format,
            camera_bind_group_layout,
            device,
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShaderKey {
    Rectangle,
    Ellipse,
}
