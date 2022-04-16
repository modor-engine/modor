use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::utils;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct OpaqueInstanceStorage {
    group_idxs: TiVec<ShaderIdx, TiVec<ModelIdx, Option<GroupIdx>>>,
    groups: TiVec<GroupIdx, Group>,
}

impl OpaqueInstanceStorage {
    pub(super) fn reset(&mut self) {
        for group in &mut self.groups {
            group.reset();
        }
    }

    pub(super) fn add(
        &mut self,
        instance: Instance,
        shader_idx: ShaderIdx,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) {
        let group_idx = self.group_idx_or_create(shader_idx, model_idx, renderer);
        self.groups[group_idx].instances.data_mut().push(instance);
    }

    pub(super) fn sync_buffers(&mut self, renderer: &Renderer) {
        for group in &mut self.groups {
            group.instances.sync(renderer);
        }
    }

    pub(super) fn render<'a>(
        &'a mut self,
        commands: &mut RenderCommands<'a>,
        shaders: &'a ShaderStorage,
        models: &'a ModelStorage,
    ) {
        let mut current_shader_idx = None;
        for group in &mut self.groups {
            if current_shader_idx != Some(group.shader_idx) {
                current_shader_idx = Some(group.shader_idx);
                commands.push_shader_change(shaders.get(group.shader_idx));
            }
            let model = models.get(group.model_idx);
            commands.push_draw(
                &model.vertex_buffer,
                &model.index_buffer,
                &group.instances,
                0..group.instances.len(),
            )
        }
    }

    fn group_idx_or_create(
        &mut self,
        shader_idx: ShaderIdx,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) -> GroupIdx {
        self.group_idxs
            .get(shader_idx)
            .and_then(|s| s.get(model_idx))
            .copied()
            .flatten()
            .unwrap_or_else(|| self.create_group(shader_idx, model_idx, renderer))
    }

    fn create_group(
        &mut self,
        shader_idx: ShaderIdx,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) -> GroupIdx {
        let group_idx = self.groups.push_and_get_key(Group {
            shader_idx,
            model_idx,
            instances: DynamicBuffer::empty(
                DynamicBufferUsage::INSTANCE,
                format!("modor_instance_buffer_opaque_{}", self.groups.len()),
                renderer,
            ),
        });
        utils::set_value(&mut self.group_idxs, shader_idx, ti_vec![]);
        utils::set_value(&mut self.group_idxs[shader_idx], model_idx, Some(group_idx));
        group_idx
    }
}

idx_type!(GroupIdx);

struct Group {
    shader_idx: ShaderIdx,
    model_idx: ModelIdx,
    instances: DynamicBuffer<Instance>,
}

impl Group {
    fn reset(&mut self) {
        self.instances.data_mut().clear();
    }
}
