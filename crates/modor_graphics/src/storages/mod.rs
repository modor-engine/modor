use self::textures::TextureKey;
use crate::backend::buffer::DynamicBuffer;
use crate::backend::data::Instance;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::TextureStorage;
use fxhash::FxHashSet;
use std::ops::Range;

pub(crate) mod core;
pub(crate) mod models;
pub(crate) mod opaque_instances;
pub(crate) mod shaders;
pub(crate) mod textures;
pub(crate) mod transparent_instances;

#[derive(Clone, PartialEq, Eq, Hash)]
struct InstanceDetails {
    shader_idx: ShaderIdx,
    texture_key: TextureKey,
    model_idx: ModelIdx,
}

#[allow(clippy::too_many_arguments)]
fn push_shape_commands<'a>(
    commands: &mut RenderCommands<'a>,
    shaders: &'a ShaderStorage,
    textures: &'a TextureStorage,
    models: &'a ModelStorage,
    instances: &'a DynamicBuffer<Instance>,
    instance_idxs: Range<usize>,
    details: &InstanceDetails,
    logged_missing_texture_keys: &mut FxHashSet<TextureKey>,
) {
    commands.push_shader_change(shaders.get(details.shader_idx));
    let key = &details.texture_key;
    let texture = textures.get(key).unwrap_or_else(|| {
        if !logged_missing_texture_keys.contains(key) {
            error!("texture with ID '{:?}' used for shape but not loaded", key);
            logged_missing_texture_keys.insert(key.clone());
        }
        textures.get_default()
    });
    commands.push_texture_binding(texture, 1);
    let model = models.get(details.model_idx);
    commands.push_draw(
        &model.vertex_buffer,
        &model.index_buffer,
        instances,
        instance_idxs,
    );
}
