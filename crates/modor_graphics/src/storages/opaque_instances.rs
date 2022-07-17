use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use modor_internal::ti_vec::TiVecSafeOperations;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct OpaqueInstanceStorage {
    instances: TiVec<ShaderIdx, TiVec<ModelIdx, Option<DynamicBuffer<Instance>>>>,
    buffer_count: usize,
}

impl OpaqueInstanceStorage {
    pub(super) fn reset(&mut self) {
        for shader_instances in &mut self.instances {
            for instances in shader_instances.iter_mut().flatten() {
                instances.data_mut().clear();
            }
        }
    }

    pub(super) fn add(
        &mut self,
        instance: Instance,
        shader_idx: ShaderIdx,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) {
        let instances = self
            .instances
            .get_mut_or_create(shader_idx)
            .get_mut_or_create(model_idx);
        if let Some(instances) = instances {
            instances.data_mut().push(instance);
        } else {
            let instance = DynamicBuffer::new(
                vec![instance],
                DynamicBufferUsage::Instance,
                format!("modor_instance_buffer_opaque_{}", self.buffer_count),
                renderer,
            );
            *instances = Some(instance);
            self.buffer_count += 1;
        }
    }

    pub(super) fn sync_buffers(&mut self, renderer: &Renderer) {
        for shader_instances in &mut self.instances {
            for instances in shader_instances.iter_mut().flatten() {
                instances.sync(renderer);
            }
        }
    }

    pub(super) fn render<'a>(
        &'a self,
        commands: &mut RenderCommands<'a>,
        shaders: &'a ShaderStorage,
        models: &'a ModelStorage,
    ) {
        for (shader_idx, shader_instances) in self.instances.iter_enumerated() {
            commands.push_shader_change(shaders.get(shader_idx));
            for (model_idx, model_instances) in shader_instances
                .iter_enumerated()
                .filter_map(|(m, i)| i.as_ref().map(|i| (m, i)))
            {
                let model = models.get(model_idx);
                commands.push_draw(
                    &model.vertex_buffer,
                    &model.index_buffer,
                    model_instances,
                    0..model_instances.len(),
                );
            }
        }
    }
}
