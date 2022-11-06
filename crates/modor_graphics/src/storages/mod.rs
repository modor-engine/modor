use crate::backend::buffer::DynamicBuffer;
use crate::backend::data::Instance;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::{TextureIdx, TextureStorage};
use modor_internal::ti_vec::TiVecSafeOperations;
use std::ops::Range;
use typed_index_collections::TiVec;

pub(crate) mod core;
pub(crate) mod models;
pub(crate) mod opaque_instances;
pub(crate) mod shaders;
pub(crate) mod textures;
pub(crate) mod transparent_instances;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct InstanceDetails {
    shader_idx: ShaderIdx,
    texture_idx: TextureIdx,
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
    details: InstanceDetails,
    logged_missing_textures: &mut TiVec<TextureIdx, bool>,
) {
    commands.push_shader_change(shaders.get(details.shader_idx));
    let texture = textures.get(details.texture_idx).unwrap_or_else(|| {
        let logged_missing_texture = logged_missing_textures.get_mut_or_create(details.texture_idx);
        if !*logged_missing_texture {
            error!(
                "texture with ID '{}' used for shape but not loaded",
                details.texture_idx.0
            );
            *logged_missing_texture = true;
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
