use crate::instances::Instance;
use crate::resources::buffers::GpuData;
use crate::resources::cameras::CameraData;
use crate::resources::models::Vertex;
use crate::resources::uniforms::Uniform;
use fxhash::FxHashMap;
use modor::{Built, Changed, Entity, EntityBuilder, Filter, Query};
use modor_internal::dyn_types::DynType;
use wgpu::{
    BindGroupLayout, BlendState, ColorTargetState, ColorWrites, CompareFunction, DepthBiasState,
    DepthStencilState, Device, FragmentState, FrontFace, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, RenderPass,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    StencilState, TextureFormat, VertexBufferLayout, VertexState,
};

pub(crate) struct ShaderRegistry {
    entity_ids: FxHashMap<ShaderKey, usize>,
}

#[singleton]
impl ShaderRegistry {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            entity_ids: FxHashMap::default(),
        })
    }

    #[run]
    fn register(&mut self, shaders: Query<'_, (&Shader, Entity<'_>, Filter<Changed<Shader>>)>) {
        for (shader, entity, _) in shaders.iter() {
            self.entity_ids.insert(shader.key.clone(), entity.id());
        }
    }

    pub(crate) fn find<'a>(
        &self,
        key: &ShaderKey,
        query: &'a Query<'_, &Shader>,
    ) -> Option<&'a Shader> {
        self.entity_ids.get(key).and_then(|&i| query.get(i))
    }
}

pub(crate) struct Shader {
    key: ShaderKey,
    pipeline: RenderPipeline,
}

#[entity]
impl Shader {
    pub(crate) const CAMERA_GROUP: u32 = 0;

    #[allow(clippy::cast_possible_truncation)]
    const VERTEX_BUFFER_LAYOUTS: &'static [VertexBufferLayout<'static>] = &[
        <Vertex as GpuData<0>>::LAYOUT,
        <Instance as GpuData<{ <Vertex as GpuData<0>>::ATTRIBUTES.len() as u32 }>>::LAYOUT,
    ];

    pub(crate) fn build_rectangle(
        target_format: TextureFormat,
        camera: &Uniform<CameraData>,
        device: &Device,
    ) -> impl Built<Self> {
        Self::build(
            ShaderKey::new(ShaderRef::Rectangle),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
            target_format,
            camera,
            device,
        )
    }

    pub(crate) fn build_ellipse(
        target_format: TextureFormat,
        camera: &Uniform<CameraData>,
        device: &Device,
    ) -> impl Built<Self> {
        Self::build(
            ShaderKey::new(ShaderRef::Ellipse),
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")),
            target_format,
            camera,
            device,
        )
    }

    fn build(
        key: ShaderKey,
        code: &str,
        target_format: TextureFormat,
        camera: &Uniform<CameraData>,
        device: &Device,
    ) -> impl Built<Self> {
        let bind_group_layouts = &[camera.bind_group_layout()];
        let pipeline_layout = Self::create_pipeline_layout(&key, bind_group_layouts, device);
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("modor_shader_{:?}", key)),
            source: ShaderSource::Wgsl(code.into()),
        });
        EntityBuilder::new(Self {
            key,
            pipeline: Self::create_pipeline(device, &pipeline_layout, target_format, &shader),
        })
    }

    pub(crate) fn use_for_rendering<'a>(&'a self, pass: &mut RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
    }

    fn create_pipeline_layout(
        key: &ShaderKey,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShaderKey(DynType);

impl ShaderKey {
    pub(crate) fn new(ref_: ShaderRef) -> Self {
        Self(DynType::new(ref_))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ShaderRef {
    Rectangle,
    Ellipse,
}
