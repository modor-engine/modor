use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use modor_internal::ti_vec;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct OpaqueInstanceStorage {
    instances: TiVec<ShaderIdx, TiVec<ModelIdx, Option<DynamicBuffer<Instance>>>>,
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
        if self.instances.get(shader_idx).is_none() {
            ti_vec::set_value(&mut self.instances, shader_idx, ti_vec![]);
        }
        if let Some(Some(instances)) = self.instances[shader_idx].get_mut(model_idx) {
            instances.data_mut().push(instance);
        } else {
            let instance = DynamicBuffer::new(
                vec![instance],
                DynamicBufferUsage::Instance,
                format!("modor_instance_buffer_opaque_{}", self.instances.len()),
                renderer,
            );
            ti_vec::set_value(&mut self.instances[shader_idx], model_idx, Some(instance));
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
