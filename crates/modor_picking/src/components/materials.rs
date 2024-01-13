use crate::entities::module::{BLANK_TEXTURE, DEFAULT_SHADER, ELLIPSE_SHADER};
use crate::PickingMaterialSource;
use bytemuck::{Pod, Zeroable};
use modor::{Entity, Not, SystemParamWithLifetime};
use modor_graphics::{Default2DMaterial, InstanceData, MaterialSource, Shader, Texture};
use modor_math::Vec2;
use modor_resources::ResKey;

#[derive(Component, NoSystem, Debug)]
pub(crate) struct Default2DPickingMaterial {
    texture_position: Vec2,
    texture_size: Vec2,
    texture_key: Option<ResKey<Texture>>,
    is_ellipse: bool,
}

impl MaterialSource for Default2DPickingMaterial {
    type Data = Default2DPickingMaterialData;
    type InstanceData = Default2DPickingMaterialInstanceData;

    fn data(&self) -> Self::Data {
        Default2DPickingMaterialData {
            texture_position: [self.texture_position.x, self.texture_position.y],
            texture_size: [self.texture_size.x, self.texture_size.y],
            has_texture: u32::from(self.texture_key.is_some()),
            padding1: 0.,
            padding2: [0., 0.],
        }
    }

    fn texture_keys(&self) -> Vec<ResKey<Texture>> {
        vec![self.texture_key.unwrap_or(BLANK_TEXTURE)]
    }

    fn shader_key(&self) -> ResKey<Shader> {
        if self.is_ellipse {
            ELLIPSE_SHADER
        } else {
            DEFAULT_SHADER
        }
    }

    fn is_transparent(&self) -> bool {
        false
    }
}

impl PickingMaterialSource for Default2DPickingMaterial {
    type Source = Default2DMaterial;

    fn convert(
        material: &Self::Source,
        render_texture_converter: impl Fn(ResKey<Texture>) -> Option<ResKey<Texture>>,
    ) -> Self {
        Self {
            texture_position: material.texture_position,
            texture_size: material.texture_size,
            texture_key: material.texture_key.and_then(render_texture_converter),
            is_ellipse: material.is_ellipse,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Zeroable, Pod)]
pub(crate) struct Default2DPickingMaterialData {
    texture_position: [f32; 2],
    texture_size: [f32; 2],
    has_texture: u32,
    padding1: f32,
    padding2: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Zeroable, Pod)]
pub(crate) struct Default2DPickingMaterialInstanceData {
    color: [f32; 4],
}

impl InstanceData for Default2DPickingMaterialInstanceData {
    type Query = Entity<'static>;
    type UpdateFilter = Not<()>; // entity ID is immutable, so no need to perform update

    #[allow(clippy::cast_possible_truncation)]
    fn data(item: <Self::Query as SystemParamWithLifetime<'_>>::Param) -> Self {
        let entity_id = [item.id() as u32];
        let color: &[u8] = bytemuck::cast_slice(&entity_id);
        Self {
            color: [
                f32::from(color[0]) / 255.,
                f32::from(color[1]) / 255.,
                f32::from(color[2]) / 255.,
                f32::from(color[3]) / 255.,
            ],
        }
    }
}
