use crate::components::managed_targets::ManagedTargets;
use crate::data::{ManagedResources, ResState};
use modor::{BuiltEntity, Custom, EntityBuilder, EntityMut, Query, SingleRef};
use modor_graphics::{Camera2D, RenderTarget};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource, ResourceRegistry};

type CameraRegistry = ResourceRegistry<Camera2D>;

#[derive(Debug, Default, SingletonComponent)]
pub(crate) struct ManagedCameras {
    pub(crate) resources: ManagedResources<ResKey<Camera2D>, Camera2D>,
}

#[systems]
impl ManagedCameras {
    #[run_after(
        component(ManagedTargets),
        component(CameraRegistry),
        component(Transform2D)
    )]
    fn update(&mut self, mut cameras: Custom<CameraAccess<'_>>) {
        let mut cameras = cameras.as_mut();
        self.resources.reset();
        self.register_resources(&mut cameras);
        self.resources
            .delete_not_registered(cameras.registry, cameras.entity.world());
        for (&key, managed_key) in self.resources.iter() {
            Self::update_resource(key, managed_key, &mut cameras);
        }
    }

    fn register_resources(&mut self, cameras: &mut CameraAccessMut<'_, '_>) {
        for camera in cameras.query.iter() {
            if camera.picking_camera.is_some() {
                continue;
            }
            if let ResState::New(key) = self.resources.register(camera.camera.key()) {
                cameras
                    .entity
                    .create_child(Self::create_resource(key, &camera, cameras));
            }
        }
    }

    fn create_resource(
        key: ResKey<Camera2D>,
        camera: &ConstCameraEntity<'_>,
        cameras: &CameraAccessMut<'_, '_>,
    ) -> impl BuiltEntity {
        let data = camera.data(cameras.managed_targets);
        EntityBuilder::new()
            .component(Camera2D::hidden(key))
            .with(|c| c.target_keys = data.managed_target_keys)
            .component(data.transform)
            .component(PickingCamera)
    }

    fn update_resource(
        key: ResKey<Camera2D>,
        managed_key: ResKey<Camera2D>,
        resources: &mut CameraAccessMut<'_, '_>,
    ) -> Option<()> {
        let camera = resources.camera(key)?;
        let data = camera.data(resources.managed_targets);
        let mut managed_camera = resources.camera_mut(managed_key)?;
        managed_camera.camera.target_keys = data.managed_target_keys;
        let managed_transform = managed_camera.transform.as_mut()?;
        **managed_transform = data.transform;
        Some(())
    }
}

#[derive(Component, NoSystem)]
struct PickingCamera;

struct CameraData {
    transform: Transform2D,
    managed_target_keys: Vec<ResKey<RenderTarget>>,
}

#[allow(unused)]
#[derive(QuerySystemParam)]
struct CameraEntity<'a> {
    camera: &'a mut Camera2D,
    transform: Option<&'a mut Transform2D>,
    picking_camera: Option<&'a PickingCamera>,
}

impl ConstCameraEntity<'_> {
    fn data(&self, managed_targets: &ManagedTargets) -> CameraData {
        CameraData {
            transform: self.transform.cloned().unwrap_or_default(),
            managed_target_keys: self
                .camera
                .target_keys
                .iter()
                .filter_map(|&target_key| managed_targets.resources.managed_key(target_key))
                .collect(),
        }
    }
}

#[derive(SystemParam)]
struct CameraAccess<'a> {
    entity: EntityMut<'a>,
    managed_targets: &'a ManagedTargets,
    registry: SingleRef<'a, 'static, ResourceRegistry<Camera2D>>,
    query: Query<'a, Custom<CameraEntity<'static>>>,
}

impl<'a> CameraAccess<'a> {
    fn as_mut<'b>(&'b mut self) -> CameraAccessMut<'a, 'b> {
        CameraAccessMut {
            entity: &mut self.entity,
            managed_targets: self.managed_targets,
            registry: self.registry.get(),
            query: &mut self.query,
        }
    }
}

struct CameraAccessMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    managed_targets: &'b ManagedTargets,
    registry: &'b ResourceRegistry<Camera2D>,
    query: &'b mut Query<'a, Custom<CameraEntity<'static>>>,
}

impl CameraAccessMut<'_, '_> {
    fn camera(&self, key: ResKey<Camera2D>) -> Option<Custom<ConstCameraEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get(id)
    }

    fn camera_mut(&mut self, key: ResKey<Camera2D>) -> Option<Custom<CameraEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get_mut(id)
    }
}
