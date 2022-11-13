use super::textures::TextureKey;
use crate::backend::buffer::{DynamicBuffer, DynamicBufferUsage};
use crate::backend::data::Instance;
use crate::backend::renderer::Renderer;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::TextureStorage;
use crate::storages::InstanceDetails;
use fxhash::FxHashSet;
use std::cmp::Ordering;

pub(super) struct TransparentInstanceStorage {
    instances: DynamicBuffer<Instance>,
    instance_details: Vec<TransparentInstanceDetails>,
    logged_missing_texture_keys: FxHashSet<TextureKey>,
}

impl TransparentInstanceStorage {
    pub(super) fn new(renderer: &Renderer) -> Self {
        Self {
            instances: DynamicBuffer::new(
                vec![],
                DynamicBufferUsage::Instance,
                "modor_instance_buffer_translucent".into(),
                renderer,
            ),
            instance_details: vec![],
            logged_missing_texture_keys: FxHashSet::default(),
        }
    }

    pub(super) fn reset(&mut self) {
        self.instances.data_mut().clear();
        self.instance_details.clear();
    }

    pub(super) fn add(
        &mut self,
        instance: Instance,
        shader_idx: ShaderIdx,
        texture_key: TextureKey,
        model_idx: ModelIdx,
    ) {
        let initial_idx = self.instance_details.len();
        self.instances.data_mut().push(instance);
        self.instance_details.push(TransparentInstanceDetails {
            initial_idx,
            inner: InstanceDetails {
                shader_idx,
                texture_key,
                model_idx,
            },
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

    pub(super) fn sync_buffers(&mut self, renderer: &Renderer) {
        self.instances.sync(renderer);
    }

    pub(super) fn render<'a>(
        &'a mut self,
        commands: &mut RenderCommands<'a>,
        shaders: &'a ShaderStorage,
        textures: &'a TextureStorage,
        models: &'a ModelStorage,
    ) {
        let mut next_instance_idx = 0;
        while let Some(details) = self.instance_details.get(next_instance_idx) {
            let first_instance_idx_with_different_model =
                self.first_instance_idx_with_config_change(next_instance_idx, details);
            super::push_shape_commands(
                commands,
                shaders,
                textures,
                models,
                &self.instances,
                next_instance_idx..first_instance_idx_with_different_model,
                &details.inner,
                &mut self.logged_missing_texture_keys,
            );
            next_instance_idx = first_instance_idx_with_different_model;
        }
    }

    fn first_instance_idx_with_config_change(
        &self,
        first_instance_idx: usize,
        current_details: &TransparentInstanceDetails,
    ) -> usize {
        for (instance_offset, details) in self.instance_details[first_instance_idx..]
            .iter()
            .enumerate()
        {
            if details.inner != current_details.inner {
                return first_instance_idx + instance_offset;
            }
        }
        self.instance_details.len()
    }
}

#[derive(Clone)]
struct TransparentInstanceDetails {
    initial_idx: usize,
    inner: InstanceDetails,
}
