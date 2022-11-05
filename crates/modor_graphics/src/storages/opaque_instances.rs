use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::{TextureIdx, TextureStorage};
use crate::storages::InstanceDetails;
use fxhash::FxHashMap;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(super) struct OpaqueInstanceStorage {
    instances: FxHashMap<InstanceDetails, DynamicBuffer<Instance>>,
    logged_missing_textures: TiVec<TextureIdx, bool>,
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
        texture_idx: TextureIdx,
        model_idx: ModelIdx,
        renderer: &Renderer,
    ) {
        let details = InstanceDetails {
            shader_idx,
            texture_idx,
            model_idx,
        };
        self.instances
            .entry(details)
            .and_modify(|i| i.data_mut().push(instance))
            .or_insert_with(|| {
                DynamicBuffer::new(
                    vec![instance],
                    DynamicBufferUsage::Instance,
                    format!(
                        "modor_instance_buffer_opaque_shader_{}_texture_{}_model_{}",
                        details.shader_idx.0, details.texture_idx.0, details.model_idx.0
                    ),
                    renderer,
                )
            });
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
        for (&details, instances) in &self.instances {
            super::push_shape_commands(
                commands,
                shaders,
                textures,
                models,
                instances,
                0..instances.len(),
                details,
                &mut self.logged_missing_textures,
            );
        }
    }
}
