use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use std::cmp::Ordering;

pub(super) struct TransparentInstanceStorage {
    instances: DynamicBuffer<Instance>,
    instance_details: Vec<InstanceDetails>,
}

impl TransparentInstanceStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        Self {
            instances: DynamicBuffer::empty(
                DynamicBufferUsage::INSTANCE,
                "modor_instance_buffer_translucent".into(),
                renderer,
            ),
            instance_details: vec![],
        }
    }

    pub(super) fn reset(&mut self) {
        self.instances.data_mut().clear();
        self.instance_details.clear();
    }

    pub(super) fn add(&mut self, instance: Instance, shader_idx: ShaderIdx, model_idx: ModelIdx) {
        let initial_idx = self.instance_details.len();
        self.instances.data_mut().push(instance);
        self.instance_details.push(InstanceDetails {
            initial_idx,
            shader_idx,
            model_idx,
        });
    }

    pub(super) fn sort(&mut self) {
        let instances = self.instances.data_mut();
        self.instance_details.sort_unstable_by(|a, b| {
            instances[b.initial_idx].transform[3][2]
                .partial_cmp(&instances[a.initial_idx].transform[3][2])
                .unwrap_or(Ordering::Equal)
        });
        self.instances.data_mut().sort_unstable_by(|a, b| {
            b.transform[3][2]
                .partial_cmp(&a.transform[3][2])
                .unwrap_or(Ordering::Equal)
        });
    }

    pub(super) fn render<'a>(
        &'a mut self,
        commands: &mut RenderCommands<'a>,
        renderer: &Renderer,
        shaders: &'a ShaderStorage,
        models: &'a ModelStorage,
    ) {
        self.instances.sync(renderer);
        let mut current_shader_idx = None;
        let mut next_instance_idx = 0;
        loop {
            if let Some(details) = self.instance_details.get(next_instance_idx) {
                let model_idx = details.model_idx;
                let first_instance_idx_with_different_model =
                    self.first_instance_idx_with_config_change(next_instance_idx, model_idx);
                let model = models.get(model_idx);
                if current_shader_idx != Some(details.shader_idx) {
                    current_shader_idx = Some(details.shader_idx);
                    commands.push_shader_change(shaders.get(details.shader_idx));
                }
                commands.push_draw(
                    &model.vertex_buffer,
                    &model.index_buffer,
                    &self.instances,
                    next_instance_idx..first_instance_idx_with_different_model,
                );
                next_instance_idx = first_instance_idx_with_different_model;
            } else {
                break;
            }
        }
    }

    fn first_instance_idx_with_config_change(
        &self,
        first_instance_idx: usize,
        model_idx: ModelIdx,
    ) -> usize {
        for (instance_offset, details) in self.instance_details[first_instance_idx..]
            .iter()
            .enumerate()
        {
            if details.model_idx != model_idx {
                return first_instance_idx + instance_offset;
            }
        }
        self.instance_details.len()
    }
}

struct InstanceDetails {
    initial_idx: usize,
    shader_idx: ShaderIdx,
    model_idx: ModelIdx,
}
