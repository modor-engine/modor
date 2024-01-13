use crate::components::managed_cameras::ManagedCameras;
use crate::components::managed_materials::ManagedMaterials;
use crate::data::{ManagedResources, ResState};
use crate::NoPicking;
use modor::{
    BuiltEntity, Custom, Entity, EntityBuilder, EntityMut, Filter, Not, Query, SingleRef, With,
};
use modor_graphics::{Camera2D, InstanceGroup2D, InstanceRendering2D, Material};
use modor_resources::{ResKey, Resource, ResourceRegistry, ResourceState};

type RenderingRegistry = ResourceRegistry<PickingRendering>;

#[derive(Default, Debug, SingletonComponent)]
pub(crate) struct ManagedRenderings {
    resources: ManagedResources<usize, PickingRendering>,
}

#[systems]
impl ManagedRenderings {
    #[run_after(
        component(ManagedCameras),
        component(ManagedMaterials),
        component(RenderingRegistry)
    )]
    fn update(&mut self, mut renderings: Custom<RenderingAccess<'_>>) {
        let mut renderings = renderings.as_mut();
        self.resources.reset();
        self.register_resources(&mut renderings);
        self.resources
            .delete_not_registered(renderings.registry, renderings.entity.world());
        for (&id, managed_key) in self.resources.iter() {
            Self::update_resource(id, managed_key, &mut renderings);
        }
    }

    fn register_resources(&mut self, renderings: &mut RenderingAccessMut<'_, '_>) {
        for rendering in renderings.query.iter() {
            if rendering.picking_rendering.is_some() {
                continue;
            }
            if let Some(data) = rendering.data(renderings) {
                if let ResState::New(key) = self.resources.register(rendering.entity.id()) {
                    renderings
                        .entity
                        .create_child(Self::create_resource(key, data));
                }
            }
        }
    }

    fn create_resource(key: ResKey<PickingRendering>, data: RenderingData) -> impl BuiltEntity {
        EntityBuilder::new()
            .component(InstanceRendering2D::new(
                data.group_key,
                data.camera_key,
                data.material_key,
            ))
            .component(PickingRendering(key))
    }
    fn update_resource(
        id: usize,
        managed_key: ResKey<PickingRendering>,
        renderings: &mut RenderingAccessMut<'_, '_>,
    ) -> Option<()> {
        let rendering = renderings.query.get(id)?;
        let data = rendering.data(renderings)?;
        let mut managed_rendering = renderings.rendering_mut(managed_key)?;
        managed_rendering.rendering.group_key = data.group_key;
        managed_rendering.rendering.camera_key = data.camera_key;
        managed_rendering.rendering.material_key = data.material_key;
        Some(())
    }
}

#[derive(Debug, Component, NoSystem)]
pub(crate) struct PickingRendering(ResKey<Self>);

impl Resource for PickingRendering {
    fn key(&self) -> ResKey<Self> {
        self.0
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}

#[allow(clippy::struct_field_names)]
struct RenderingData {
    group_key: ResKey<InstanceGroup2D>,
    camera_key: ResKey<Camera2D>,
    material_key: ResKey<Material>,
}

#[allow(unused)]
#[derive(QuerySystemParam)]
struct RenderingEntity<'a> {
    entity: Entity<'a>,
    rendering: &'a mut InstanceRendering2D,
    picking_rendering: Option<&'a PickingRendering>,
    _filter: Filter<Not<With<NoPicking>>>,
}

impl ConstRenderingEntity<'_> {
    fn data(&self, renderings: &RenderingAccessMut<'_, '_>) -> Option<RenderingData> {
        Some(RenderingData {
            group_key: self.rendering.group_key,
            camera_key: renderings
                .managed_cameras
                .resources
                .managed_key(self.rendering.camera_key)?,
            material_key: renderings
                .managed_materials
                .resources
                .managed_key(self.rendering.material_key)?,
        })
    }
}

#[derive(SystemParam)]
struct RenderingAccess<'a> {
    entity: EntityMut<'a>,
    managed_materials: &'a ManagedMaterials,
    managed_cameras: &'a ManagedCameras,
    registry: SingleRef<'a, 'static, ResourceRegistry<PickingRendering>>,
    query: Query<'a, Custom<RenderingEntity<'static>>>,
}

impl<'a> RenderingAccess<'a> {
    fn as_mut<'b>(&'b mut self) -> RenderingAccessMut<'a, 'b> {
        RenderingAccessMut {
            entity: &mut self.entity,
            managed_materials: self.managed_materials,
            managed_cameras: self.managed_cameras,
            registry: self.registry.get(),
            query: &mut self.query,
        }
    }
}

struct RenderingAccessMut<'a, 'b> {
    entity: &'b mut EntityMut<'a>,
    managed_materials: &'b ManagedMaterials,
    managed_cameras: &'b ManagedCameras,
    registry: &'b ResourceRegistry<PickingRendering>,
    query: &'b mut Query<'a, Custom<RenderingEntity<'static>>>,
}

impl RenderingAccessMut<'_, '_> {
    fn rendering_mut(
        &mut self,
        key: ResKey<PickingRendering>,
    ) -> Option<Custom<RenderingEntity<'_>>> {
        let id = self.registry.entity_id(key)?;
        self.query.get_mut(id)
    }
}
