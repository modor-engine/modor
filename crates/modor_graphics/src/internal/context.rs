use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::{GpuData, Instance, Vertex};
use crate::backend::renderer::Renderer;
use crate::backend::rendering::{RenderCommands, Rendering};
use crate::backend::shaders::Shader;
use crate::{GraphicsModule, ShapeColor};
use modor::{Built, EntityBuilder, Query, Single};
use modor_physics::{Position, Scale, Shape};
use std::cmp::Ordering;
use typed_index_collections::TiVec;

// TODO: should Scale be relative ? (issue: complicated to create squares as child entity)

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
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/rectangle.wgsl")),
            &[
                <Vertex as GpuData<0>>::layout(),
                <Instance as GpuData<1>>::layout(),
            ],
            "rectangle",
            &renderer,
        );
        let circle_shader = Shader::new(
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/circle.wgsl")),
            &[
                <Vertex as GpuData<0>>::layout(),
                <Instance as GpuData<1>>::layout(),
            ],
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
                    DynamicBufferUsage::INSTANCE,
                    "modor_instance_buffer_opaque_rectangle".into(),
                    &renderer,
                ),
                DynamicBuffer::empty(
                    DynamicBufferUsage::INSTANCE,
                    "modor_instance_buffer_opaque_circle".into(),
                    &renderer,
                )
            ],
            sorted_translucent_instances: DynamicBuffer::empty(
                DynamicBufferUsage::INSTANCE,
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
        let window_size = self.renderer.window_size();
        let x_scale = if window_size.width > window_size.height {
            window_size.height as f32 / window_size.width as f32
        } else {
            1.
        };
        let y_scale = if window_size.width > window_size.height {
            1.
        } else {
            window_size.width as f32 / window_size.height as f32
        };
        let translucent_instances = self.sorted_translucent_instances.data_mut();
        for instances in &mut self.opaque_instances {
            instances.data_mut().clear();
        }
        translucent_instances.clear();
        self.translucent_instance_metadata.clear();
        let (min_z, max_z) = shapes
            .iter()
            .map(|(_, p, _, _)| p.z)
            .fold((f32::INFINITY, 0_f32), |(min, max), b| {
                (min.min(b), max.max(b))
            });
        for (color, position, scale, shape) in shapes.iter() {
            let instance =
                Self::create_instance(color, position, scale, min_z, max_z, x_scale, y_scale);
            let model_idx = match shape.unwrap_or(&Shape::Rectangle2D) {
                Shape::Rectangle2D => ModelIdx::from(0),
                Shape::Circle2D => ModelIdx::from(1),
            };
            if color.0.a > 0. && color.0.a < 1. {
                translucent_instances.push(instance);
                self.translucent_instance_metadata
                    .push(TranslucentInstanceMetadata {
                        initial_idx: self.translucent_instance_metadata.len(),
                        model_idx,
                    });
            } else {
                self.opaque_instances[model_idx].data_mut().push(instance);
            }
        }
        self.translucent_instance_metadata.sort_unstable_by(|a, b| {
            translucent_instances[b.initial_idx].transform[3][2]
                .partial_cmp(&translucent_instances[a.initial_idx].transform[3][2])
                .unwrap_or(Ordering::Equal)
        });
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
        let mut next_instance_idx = 0;
        loop {
            if let Some(metadata) = self.translucent_instance_metadata.get(next_instance_idx) {
                let model_idx = metadata.model_idx;
                let first_instance_idx_with_different_model =
                    self.first_instance_idx_with_different_model(model_idx, next_instance_idx);
                let model = &self.models[model_idx];
                if current_shader_idx != Some(model.shader_idx) {
                    current_shader_idx = Some(model.shader_idx);
                    commands.push_shader_change(&self.shaders[model.shader_idx]);
                }
                commands.push_draw(
                    &model.vertex_buffer,
                    &model.index_buffer,
                    &self.sorted_translucent_instances,
                    next_instance_idx..first_instance_idx_with_different_model,
                );
                next_instance_idx = first_instance_idx_with_different_model;
            } else {
                break;
            }
        }
        drop(commands);
        rendering.apply();
    }

    fn create_instance(
        c: &ShapeColor,
        p: &Position,
        s: Option<&Scale>,
        min_z: f32,
        max_z: f32,
        x_scale: f32,
        y_scale: f32,
    ) -> Instance {
        let scale = s.unwrap_or(&DEFAULT_SCALE);
        let z_position = (1. - (p.z - min_z) / (max_z - min_z)) * MAX_2D_DEPTH;
        Instance {
            transform: [
                [scale.x * 2. * x_scale, 0., 0., 0.],
                [0., scale.y * 2. * y_scale, 0., 0.],
                [0., 0., 0., 0.],
                [p.x * 2. * x_scale, p.y * 2. * y_scale, z_position, 1.],
            ],
            color: [c.0.r, c.0.g, c.0.b, c.0.a],
        }
    }

    fn first_instance_idx_with_different_model(
        &self,
        model_idx: ModelIdx,
        first_instance_idx: usize,
    ) -> usize {
        for (instance_offset, metadata) in self.translucent_instance_metadata[first_instance_idx..]
            .iter()
            .enumerate()
        {
            if metadata.model_idx != model_idx {
                return first_instance_idx + instance_offset;
            }
        }
        self.translucent_instance_metadata.len()
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
                DynamicBufferUsage::VERTEX,
                format!("modor_vertex_buffer_{}", label),
                renderer,
            ),
            index_buffer: DynamicBuffer::new(
                indices,
                DynamicBufferUsage::INDEX,
                format!("modor_index_buffer_{}", label),
                renderer,
            ),
        }
    }
}

idx_type!(ShaderIdx);
idx_type!(ModelIdx);

#[derive(Debug)]
struct TranslucentInstanceMetadata {
    initial_idx: usize,
    model_idx: ModelIdx,
}
