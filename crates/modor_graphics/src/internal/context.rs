use crate::backend::buffer::DynamicBuffer;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::backend::shaders::Shader;
use crate::{GraphicsModule, ShapeColor};
use modor::{Built, EntityBuilder, Query, Single};
use modor_physics::{Position, Scale, Shape};
use std::cmp::Ordering;
use std::mem;
use typed_index_collections::TiVec;
use wgpu::{
    include_wgsl, vertex_attr_array, BufferAddress, BufferUsages, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};

const DEFAULT_SCALE: Scale = Scale::xyz(1., 1., 1.);
const MAX_2D_DEPTH: f32 = 0.9; // used to fix shape disappearance when depth is near to 1
const RECTANGLE_VERTICES: [Vertex; 4] = [
    Vertex {
        position: [-0.5, 0.5, 0.],
    },
    Vertex {
        position: [-0.5, -0.5, 0.],
    },
    Vertex {
        position: [0.5, -0.5, 0.],
    },
    Vertex {
        position: [0.5, 0.5, 0.],
    },
];
const RECTANGLE_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

pub(crate) struct Context {
    renderer: Renderer,
    shaders: TiVec<ShaderIdx, Shader>,
    models: TiVec<ModelIdx, Model>,
    opaque_instances: TiVec<ModelIdx, DynamicBuffer<Instance>>,
    sorted_translucent_instances: DynamicBuffer<Instance>,
    translucent_instance_metadata: Vec<TranslucentInstanceMetadata>,
}

#[singleton]
impl Context {
    pub(crate) fn build(renderer: Renderer) -> impl Built<Self> {
        let rectangle_shader = Shader::new(
            include_wgsl!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
            &[Vertex::layout(), Instance::layout()],
            "rectangle",
            &renderer,
        );
        let circle_shader = Shader::new(
            include_wgsl!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/circle.wgsl")),
            &[Vertex::layout(), Instance::layout()],
            "circle",
            &renderer,
        );
        let rectangle_model = Model::new(
            ShaderIdx::from(0),
            "rectangle",
            RECTANGLE_VERTICES.to_vec(),
            RECTANGLE_INDICES.to_vec(),
            &renderer,
        );
        let circle_model = Model::new(
            ShaderIdx::from(1),
            "circle",
            RECTANGLE_VERTICES.to_vec(),
            RECTANGLE_INDICES.to_vec(),
            &renderer,
        );
        EntityBuilder::new(Self {
            shaders: ti_vec![rectangle_shader, circle_shader],
            models: ti_vec![rectangle_model, circle_model],
            opaque_instances: ti_vec![
                DynamicBuffer::empty(
                    BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    "modor_instance_buffer_opaque_rectangle".into(),
                    &renderer,
                ),
                DynamicBuffer::empty(
                    BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    "modor_instance_buffer_opaque_circle".into(),
                    &renderer,
                )
            ],
            sorted_translucent_instances: DynamicBuffer::empty(
                BufferUsages::VERTEX | BufferUsages::COPY_DST,
                "modor_instance_buffer_translucent".into(),
                &renderer,
            ),
            translucent_instance_metadata: vec![],
            renderer,
        })
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    #[run]
    pub(crate) fn update(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        for instances in &mut self.opaque_instances {
            instances.data_mut().clear();
        }
        self.sorted_translucent_instances.data_mut().clear();
        let (min_z, max_z) = shapes
            .iter()
            .map(|(_, p, _, _)| p.z)
            .fold((f32::INFINITY, 0_f32), |(min, max), b| {
                (min.min(b), max.max(b))
            });
        for (color, position, scale, shape) in shapes.iter() {
            let instance = Self::create_instance(color, position, scale, min_z, max_z);
            if color.0.a > 0. && color.0.a < 1. {
                self.sorted_translucent_instances.data_mut().push(instance);
            } else {
                let model_idx = match shape.unwrap_or(&Shape::Rectangle2D) {
                    Shape::Rectangle2D => ModelIdx::from(0),
                    Shape::Circle2D => ModelIdx::from(1),
                };
                self.opaque_instances[model_idx].data_mut().push(instance);
            }
        }
        self.sorted_translucent_instances
            .data_mut()
            .sort_unstable_by(|a, b| {
                b.transform[3][2]
                    .partial_cmp(&a.transform[3][2])
                    .unwrap_or(Ordering::Equal)
            });
    }

    #[run_after_previous]
    fn render(&mut self, module: Single<'_, GraphicsModule>) {
        for instances in &mut self.opaque_instances {
            instances.sync(&self.renderer);
        }
        self.sorted_translucent_instances.sync(&self.renderer);
        let mut rendering = Rendering::new(&self.renderer);
        let mut commands = RenderCommands::new(module.background_color().into(), &mut rendering);
        let mut current_shader_idx = None;
        for (model_idx, instances) in self.opaque_instances.iter_enumerated() {
            let model = &self.models[model_idx];
            if current_shader_idx != Some(model.shader_idx) {
                current_shader_idx = Some(model.shader_idx);
                commands.push_shader_change(&self.shaders[model.shader_idx]);
            }
            commands.push_draw(
                &model.vertex_buffer,
                &model.index_buffer,
                instances,
                0..instances.len(),
            )
        }
        // TODO: handle transparency correctly
        commands.push_shader_change(&self.shaders[ShaderIdx::from(0)]);
        commands.push_draw(
            &self.models[ModelIdx::from(0)].vertex_buffer,
            &self.models[ModelIdx::from(0)].index_buffer,
            &self.sorted_translucent_instances,
            0..self.sorted_translucent_instances.len(),
        );
        drop(commands);
        rendering.apply();
    }

    fn create_instance(
        c: &ShapeColor,
        p: &Position,
        s: Option<&Scale>,
        min_z: f32,
        max_z: f32,
    ) -> Instance {
        let scale = s.unwrap_or(&DEFAULT_SCALE);
        let z_position = (1. - (p.z - min_z) / (max_z - min_z)) * MAX_2D_DEPTH;
        Instance {
            transform: [
                [scale.x, 0., 0., 0.],
                [0., scale.y, 0., 0.],
                [0., 0., scale.z, 0.],
                [p.x, p.y, z_position, 1.],
            ],
            color: [c.0.r, c.0.g, c.0.b, c.0.a],
        }
    }
}

struct Model {
    shader_idx: ShaderIdx,
    vertex_buffer: DynamicBuffer<Vertex>,
    index_buffer: DynamicBuffer<u16>,
}

impl Model {
    fn new(
        shader_idx: ShaderIdx,
        label: &str,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        renderer: &Renderer,
    ) -> Self {
        Self {
            shader_idx,
            vertex_buffer: DynamicBuffer::new(
                vertices,
                BufferUsages::VERTEX,
                format!("modor_vertex_buffer_{}", label),
                renderer,
            ),
            index_buffer: DynamicBuffer::new(
                indices,
                BufferUsages::INDEX,
                format!("modor_index_buffer_{}", label),
                renderer,
            ),
        }
    }
}

idx_type!(ShaderIdx);
idx_type!(ModelIdx);

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [VertexAttribute; 1] = vertex_attr_array![0 => Float32x3];

    fn layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    transform: [[f32; 4]; 4],
    color: [f32; 4],
}

impl Instance {
    const ATTRIBUTES: [VertexAttribute; 5] = vertex_attr_array![
        1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4
    ];

    fn layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

struct TranslucentInstanceMetadata {
    model: ModelIdx,
}
