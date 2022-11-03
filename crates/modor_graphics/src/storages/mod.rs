use crate::backend::buffer::DynamicBuffer;
use crate::backend::data::Instance;
use crate::backend::rendering::RenderCommands;
use crate::storages::models::{ModelIdx, ModelStorage};
use crate::storages::shaders::{ShaderIdx, ShaderStorage};
use crate::storages::textures::{TextureIdx, TextureStorage};
use log::error;
use std::ops::Range;

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
    is_missing_texture_logged: &mut bool,
) {
    commands.push_shader_change(shaders.get(details.shader_idx));
    let texture = textures.get(details.texture_idx).unwrap_or_else(|| {
        if !*is_missing_texture_logged {
            error!("texture used for shape but not loaded");
            *is_missing_texture_logged = true;
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
