use crate::components::picking::PickingTargetData;
use modor::{
    BuiltEntity, Custom, Entity, EntityBuilder, EntityMut, Filter, Query, SingleRef, With,
};
use modor_graphics::{
    Color, RenderTarget, Size, Texture, TextureBuffer, TextureBufferPart, TextureSource, Window,
};
use modor_resources::{ResKey, Resource, ResourceRegistry};

const PICKING_RENDERING: &str = "PICKING";

#[derive(SystemParam)]
pub(crate) struct TargetUpdateResource<'a> {
    entity: EntityMut<'a>,
    target_registry: SingleRef<'a, 'static, ResourceRegistry<RenderTarget>>,
    targets: Query<'a, Custom<TargetEntity<'static>>>,
}

impl<'a> TargetUpdateResource<'a> {
    pub(crate) fn as_mut<'b>(&'b mut self) -> TargetUpdateResourceMut<'a, 'b> {
        TargetUpdateResourceMut {
            entity: &mut self.entity,
            target_registry: self.target_registry.get(),
            targets: &mut self.targets,
        }
    }
}

pub(crate) struct TargetUpdateResourceMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    target_registry: &'b ResourceRegistry<RenderTarget>,
    targets: &'b mut Query<'a, Custom<TargetEntity<'static>>>,
}

impl TargetUpdateResourceMut<'_, '_> {
    pub(crate) fn update_target(
        &mut self,
        src_key: ResKey<RenderTarget>,
        data: &PickingTargetData,
    ) -> Option<()> {
        let size = self.target_size(src_key)?;
        if let Some(mut target) = self.target_mut(data.target_key) {
            if let Some(texture) = &mut target.texture {
                if texture.size() != Some(size) {
                    texture.set_source(TextureSource::Size(size));
                }
            } else {
                let target_id = target.entity.id();
                let texture = Texture::from_size(data.texture_key, size);
                self.entity.world().add_component(target_id, texture);
            }
        } else {
            let target = Self::picking_target(data, size);
            self.entity.create_child(target);
        }
        Some(())
    }

    fn target_size(&mut self, key: ResKey<RenderTarget>) -> Option<Size> {
        let id = self.target_registry.entity_id(key)?;
        let target = self.targets.get(id)?;
        target
            .window
            .map(|w| w.size())
            .or_else(|| target.texture.and_then(|t| t.size()))
    }

    fn target_mut(&mut self, key: ResKey<RenderTarget>) -> Option<Custom<TargetEntity<'_>>> {
        let id = self.target_registry.entity_id(key)?;
        self.targets.get_mut(id)
    }

    fn picking_target(data: &PickingTargetData, size: Size) -> impl BuiltEntity {
        EntityBuilder::new()
            .component(RenderTarget::new(data.target_key))
            .with(|t| t.is_anti_aliasing_enabled = false)
            .with(|t| t.background_color = Color::WHITE)
            .with(|t| t.category = PICKING_RENDERING)
            .component(Texture::from_size(data.texture_key, size))
            .component(TextureBuffer::default())
            .with(|b| b.part = TextureBufferPart::Pixels(vec![]))
    }
}

#[allow(unused)]
#[derive(QuerySystemParam)]
pub(crate) struct TargetEntity<'a> {
    entity: Entity<'a>,
    window: Option<&'a Window>,
    texture: Option<&'a mut Texture>,
    filter: Filter<With<RenderTarget>>,
}
