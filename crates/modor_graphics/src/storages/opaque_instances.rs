use super::textures::TextureKey;
use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::TextureStorage;
use crate::storages::InstanceDetails;
use fxhash::{FxHashMap, FxHashSet};

#[derive(Default)]
pub(super) struct OpaqueInstanceStorage {
    instances: FxHashMap<InstanceDetails, DynamicBuffer<Instance>>,
    logged_missing_texture_keys: FxHashSet<TextureKey>,
}

impl OpaqueInstanceStorage {
    pub(super) fn reset(&mut self) {
        for instances in self.instances.values_mut() {
            instances.data_mut().clear();
        }
    }

    pub(super) fn add(
        &mut self,
        instance: Instance,
        shader_idx: ShaderIdx,
        texture_key: TextureKey,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) {
        let details = InstanceDetails {
            shader_idx,
            texture_key,
            model_idx,
        };
        if let Some(instances) = self.instances.get_mut(&details) {
            instances.data_mut().push(instance);
        } else {
            let label = format!(
                "modor_instance_buffer_opaque_shader_{}_texture_{:?}_model_{}",
                details.shader_idx.0, details.texture_key, details.model_idx.0
            );
            self.instances.insert(
                details,
                DynamicBuffer::new(
                    vec![instance],
                    DynamicBufferUsage::Instance,
                    label,
                    renderer,
                ),
            );
        }
    }

    pub(super) fn sync_buffers(&mut self, renderer: &Renderer) {
        for instances in self.instances.values_mut() {
            instances.sync(renderer);
        }
    }

    pub(super) fn render<'a>(
        &'a mut self,
        commands: &mut RenderCommands<'a>,
        shaders: &'a ShaderStorage,
        textures: &'a TextureStorage,
        models: &'a ModelStorage,
    ) {
        for (details, instances) in &self.instances {
            super::push_shape_commands(
                commands,
                shaders,
                textures,
                models,
                instances,
                0..instances.len(),
                details,
                &mut self.logged_missing_texture_keys,
            );
        }
    }
}
