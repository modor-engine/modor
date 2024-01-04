use crate::components::picking::PickingRenderingData;
use modor::{BuiltEntity, Custom, EntityBuilder, EntityMut, Query, SingleRef};
use modor_graphics::{InstanceRendering2D, RenderTarget};
use modor_resources::{IndexResKey, ResKey, Resource, ResourceRegistry, ResourceState};

const PICKING_RENDERING: IndexResKey<PickingRendering> = IndexResKey::new("picking(modor_picking)");

#[derive(SystemParam)]
pub(crate) struct RenderingUpdateResource<'a> {
    entity: EntityMut<'a>,
    rendering_registry: SingleRef<'a, 'static, ResourceRegistry<PickingRendering>>,
    renderings: Query<'a, Custom<RenderingEntity<'static>>>,
}

impl<'a> RenderingUpdateResource<'a> {
    pub(crate) fn as_mut<'b>(&'b mut self) -> RenderingUpdateResourceMut<'a, 'b> {
        RenderingUpdateResourceMut {
            entity: &mut self.entity,
            rendering_registry: self.rendering_registry.get(),
            renderings: &mut self.renderings,
        }
    }
}

pub(crate) struct RenderingUpdateResourceMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    rendering_registry: &'b ResourceRegistry<PickingRendering>,
    renderings: &'b mut Query<'a, Custom<RenderingEntity<'static>>>,
}

impl RenderingUpdateResourceMut<'_, '_> {
    pub(crate) fn update_rendering(&mut self, data: &PickingRenderingData, index: usize) {
        let key = PICKING_RENDERING.get(index);
        if let Some(mut rendering) = self.rendering_mut(key) {
            rendering.rendering.group_key = data.group_key;
            rendering.rendering.camera_key = data.camera_key;
            rendering.rendering.material_key = data.material_key;
        } else {
            self.entity.create_child(Self::picking_rendering(data, key));
        }
    }

    fn rendering_mut(
        &mut self,
        key: ResKey<PickingRendering>,
    ) -> Option<Custom<RenderingEntity<'_>>> {
        let id = self.rendering_registry.entity_id(key)?;
        self.renderings.get_mut(id)
    }

    fn picking_rendering(
        data: &PickingRenderingData,
        key: ResKey<PickingRendering>,
    ) -> impl BuiltEntity {
        EntityBuilder::new()
            .component(InstanceRendering2D::new(
                data.group_key,
                data.camera_key,
                data.material_key,
            ))
            .component(PickingRendering(key))
    }
}

#[derive(QuerySystemParam)]
pub(crate) struct RenderingEntity<'a> {
    rendering: &'a mut InstanceRendering2D,
}

#[derive(Component, NoSystem)]
pub(crate) struct PickingRendering(ResKey<Self>);

impl Resource for PickingRendering {
    fn key(&self) -> ResKey<Self> {
        self.0
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}
