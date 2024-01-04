use crate::components::picking::PickingCameraData;
use modor::{BuiltEntity, Custom, Entity, EntityBuilder, EntityMut, Query, SingleRef};
use modor_graphics::Camera2D;
use modor_physics::Transform2D;
use modor_resources::{ResKey, ResourceRegistry};

#[derive(SystemParam)]
pub(crate) struct CameraUpdateResource<'a> {
    entity: EntityMut<'a>,
    camera_registry: SingleRef<'a, 'static, ResourceRegistry<Camera2D>>,
    cameras: Query<'a, Custom<CameraEntity<'static>>>,
}

impl<'a> CameraUpdateResource<'a> {
    pub(crate) fn as_mut<'b>(&'b mut self) -> CameraUpdateResourceMut<'a, 'b> {
        CameraUpdateResourceMut {
            entity: &mut self.entity,
            camera_registry: self.camera_registry.get(),
            cameras: &mut self.cameras,
        }
    }
}

pub(crate) struct CameraUpdateResourceMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    camera_registry: &'b ResourceRegistry<Camera2D>,
    cameras: &'b mut Query<'a, Custom<CameraEntity<'static>>>,
}

impl CameraUpdateResourceMut<'_, '_> {
    pub(crate) fn update_camera(
        &mut self,
        src_key: ResKey<Camera2D>,
        data: &PickingCameraData,
    ) -> Option<()> {
        let transform = self.camera_transform(src_key)?;
        if let Some(mut camera) = self.camera_mut(data.camera_key) {
            let target_keys = &mut camera.camera.target_keys;
            target_keys.clear();
            target_keys.extend_from_slice(&data.target_keys);
            if let Some(picking_transform) = &mut camera.transform {
                **picking_transform = transform;
            } else {
                let camera_id = camera.entity.id();
                self.entity.world().add_component(camera_id, transform);
            }
        } else {
            let camera = Self::picking_camera(data, transform);
            self.entity.create_child(camera);
        }
        Some(())
    }

    fn camera_transform(&mut self, key: ResKey<Camera2D>) -> Option<Transform2D> {
        let id = self.camera_registry.entity_id(key)?;
        let camera = self.cameras.get(id)?;
        Some(camera.transform.cloned().unwrap_or_default())
    }

    fn camera_mut(&mut self, key: ResKey<Camera2D>) -> Option<Custom<CameraEntity<'_>>> {
        let id = self.camera_registry.entity_id(key)?;
        self.cameras.get_mut(id)
    }

    fn picking_camera(data: &PickingCameraData, transform: Transform2D) -> impl BuiltEntity {
        EntityBuilder::new()
            .component(Camera2D::hidden(data.camera_key))
            .with(|c| c.target_keys = data.target_keys.clone())
            .component(transform)
    }
}

#[allow(unused)]
#[derive(QuerySystemParam)]
pub(crate) struct CameraEntity<'a> {
    entity: Entity<'a>,
    camera: &'a mut Camera2D,
    transform: Option<&'a mut Transform2D>,
}
