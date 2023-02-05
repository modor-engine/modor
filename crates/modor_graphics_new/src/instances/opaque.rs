use crate::instances::Instance;
use crate::instances::ResourceKeys;
use crate::resources::buffers::{DynamicBuffer, DynamicBufferUsage};
use crate::targets::GpuDevice;
use crate::Mesh2D;
use fxhash::{FxBuildHasher, FxHashMap};
use modor::{Built, Changed, Entity, EntityBuilder, Filter, Or, Query, Single, World};
use modor_physics::{PhysicsModule, Transform2D};
use std::collections::HashMap;
use std::iter;
use wgpu::Device;

pub(crate) struct OpaqueInstanceManager {
    latest_mesh_resources: FxHashMap<usize, ResourceKeys>,
}

#[singleton]
impl OpaqueInstanceManager {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            latest_mesh_resources: FxHashMap::default(),
        })
    }

    #[run_after(component(PhysicsModule))]
    fn delete_meshes(&mut self, mut buffers: Query<'_, &mut OpaqueInstances>, world: World<'_>) {
        let mut buffers = Self::map_buffers(&mut buffers);
        for id in world
            .transformed_entity_ids()
            .chain(world.deleted_entity_ids())
        {
            if let Some(resource_key) = self.latest_mesh_resources.remove(&id) {
                buffers
                    .get_mut(&resource_key)
                    .expect("internal error: missing instance buffer")
                    .delete(id);
            }
        }
    }

    #[run_after_previous]
    #[allow(clippy::option_if_let_else)]
    fn update_meshes(
        &mut self,
        entity: Entity<'_>,
        device: Single<'_, GpuDevice>,
        meshes: Query<'_, (&Transform2D, &Mesh2D, Entity<'_>, Filter<ChangedMesh2D>)>,
        mut buffers: Query<'_, &mut OpaqueInstances>,
        mut world: World<'_>,
    ) {
        let mut existing_buffers = Self::map_buffers(&mut buffers);
        let mut new_buffers = FxHashMap::<ResourceKeys, BufferDetails>::default();
        for (transform, mesh, entity, _) in meshes.iter() {
            self.delete_mesh(&mut existing_buffers, entity.id());
            let instance = Instance::new(transform, mesh);
            if let Some(buffer) = existing_buffers.get_mut(&mesh.resource_keys) {
                buffer.add(entity.id(), instance);
            } else if let Some(buffer) = new_buffers.get_mut(&mesh.resource_keys) {
                buffer.as_mut().add(entity.id(), instance);
            } else {
                let buffer = BufferDetails::with_mesh(entity.id(), instance);
                new_buffers.insert(mesh.resource_keys.clone(), buffer);
            }
            self.latest_mesh_resources
                .insert(entity.id(), mesh.resource_keys.clone());
        }
        Self::create_buffers(new_buffers, &device.device, entity.id(), &mut world);
    }

    fn map_buffers<'a>(
        buffers: &'a mut Query<'_, &mut OpaqueInstances>,
    ) -> FxHashMap<ResourceKeys, BufferDetailsMut<'a>> {
        buffers
            .iter_mut()
            .map(|b| (b.resource_keys.clone(), b.as_mut()))
            .collect::<FxHashMap<_, _>>()
    }

    fn create_buffers(
        buffers: HashMap<ResourceKeys, BufferDetails, FxBuildHasher>,
        device: &Device,
        entity_id: usize,
        world: &mut World<'_>,
    ) {
        for (resource_key, buffer) in buffers {
            world.create_child_entity(
                entity_id,
                OpaqueInstances::build(
                    resource_key,
                    buffer.instances,
                    buffer.mesh_positions,
                    device,
                ),
            );
        }
    }

    fn delete_mesh(
        &mut self,
        buffers: &mut FxHashMap<ResourceKeys, BufferDetailsMut<'_>>,
        mesh_id: usize,
    ) {
        if let Some(resource_key) = self.latest_mesh_resources.get_mut(&mesh_id) {
            buffers
                .get_mut(resource_key)
                .expect("internal error: missing instance buffer")
                .delete(mesh_id);
        }
    }
}

pub(crate) struct OpaqueInstances {
    resource_keys: ResourceKeys,
    mesh_positions: FxHashMap<usize, usize>,
    buffer: DynamicBuffer<Instance>,
}

#[entity]
impl OpaqueInstances {
    pub(crate) fn build(
        resource_keys: ResourceKeys,
        instances: Vec<Instance>,
        mesh_positions: FxHashMap<usize, usize>,
        device: &Device,
    ) -> impl Built<Self> {
        let label = format!("{:?}", resource_keys);
        EntityBuilder::new(Self {
            resource_keys,
            mesh_positions,
            buffer: DynamicBuffer::new(instances, DynamicBufferUsage::Instance, label, device),
        })
    }

    #[run_after(component(OpaqueInstanceManager))]
    fn sync_buffer(&mut self, device: Single<'_, GpuDevice>) {
        self.buffer.sync(&device);
    }

    pub(crate) fn resource_keys(&self) -> &ResourceKeys {
        &self.resource_keys
    }

    pub(crate) fn buffer(&self) -> &DynamicBuffer<Instance> {
        &self.buffer
    }

    fn as_mut(&mut self) -> BufferDetailsMut<'_> {
        BufferDetailsMut {
            instances: &mut self.buffer,
            mesh_positions: &mut self.mesh_positions,
        }
    }
}

#[derive(Default)]
struct BufferDetails {
    instances: Vec<Instance>,
    mesh_positions: FxHashMap<usize, usize>,
}

impl BufferDetails {
    fn with_mesh(id: usize, instance: Instance) -> Self {
        Self {
            instances: vec![instance],
            mesh_positions: iter::once((id, 0)).collect(),
        }
    }

    fn as_mut(&mut self) -> BufferDetailsMut<'_> {
        BufferDetailsMut {
            instances: &mut self.instances,
            mesh_positions: &mut self.mesh_positions,
        }
    }
}

struct BufferDetailsMut<'a> {
    instances: &'a mut Vec<Instance>,
    mesh_positions: &'a mut FxHashMap<usize, usize>,
}

impl BufferDetailsMut<'_> {
    fn add(&mut self, mesh_id: usize, instance: Instance) {
        if let Some(&position) = self.mesh_positions.get(&mesh_id) {
            self.instances[position] = instance;
        } else {
            self.mesh_positions.insert(mesh_id, self.instances.len());
            self.instances.push(instance);
        }
    }

    fn delete(&mut self, mesh_id: usize) {
        if let Some(position) = self.mesh_positions.remove(&mesh_id) {
            self.instances.swap_remove(position);
        }
    }
}

type ChangedMesh2D = Or<(Changed<Transform2D>, Changed<Mesh2D>)>;
