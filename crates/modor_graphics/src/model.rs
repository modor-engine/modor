use crate::buffer::Buffer;
use crate::gpu::{Gpu, GpuHandle};
use crate::mesh::MeshGlob;
use crate::vertex_buffer::VertexBuffer;
use crate::{Camera2DGlob, GraphicsResources, Mat, Material, MaterialGlob};
use derivative::Derivative;
use fxhash::FxHashMap;
use modor::{Context, Glob, GlobRef, Globals, NoVisit, Node, RootNode, RootNodeHandle};
use modor_input::modor_math::{Mat4, Quat, Vec2};
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem;
use wgpu::{vertex_attr_array, BufferUsages, VertexAttribute, VertexStepMode};

#[derive(Derivative, NoVisit)]
#[derivative(Debug(bound = ""))]
pub struct Model2D<T> {
    pub position: Vec2,
    pub size: Vec2,
    pub rotation: f32,
    pub z_index: i16,
    pub camera: GlobRef<Camera2DGlob>,
    material: GlobRef<Option<MaterialGlob>>,
    mesh: GlobRef<Option<MeshGlob>>,
    glob: Glob<Model2DGlob>,
    groups: RootNodeHandle<InstanceGroups2D>,
    phantom: PhantomData<fn(T)>,
}

impl<T> Node for Model2D<T>
where
    T: Material,
{
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let group = InstanceGroup2DKey::new(self);
        let old_group = mem::replace(&mut self.glob.get_mut(ctx).group, group);
        let data = T::instance_data(ctx, self.glob());
        self.groups.get_mut(ctx).update_model(self, data, old_group);
    }
}

impl<T> Model2D<T>
where
    T: Material,
{
    pub fn new(ctx: &mut Context<'_>, position: Vec2, size: Vec2) -> Self {
        let resources = ctx.root::<GraphicsResources>().get(ctx);
        let camera = resources.window_camera.glob().clone();
        let material = resources.white_material.glob().clone();
        let mesh = resources.rectangle_mesh.glob().clone();
        let model = Self {
            position,
            size,
            rotation: 0.,
            z_index: 0,
            glob: Glob::new(
                ctx,
                Model2DGlob::new(InstanceGroup2DKey {
                    mesh: mesh.index(),
                    camera: camera.index(),
                    material: material.index(),
                }),
            ),
            camera,
            material,
            mesh,
            groups: ctx.root::<InstanceGroups2D>(),
            phantom: PhantomData,
        };
        let data = T::instance_data(ctx, model.glob());
        model.groups.get_mut(ctx).register_model(&model, data);
        model
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Model2DGlob> {
        self.glob.as_ref()
    }

    // TODO: use GlobRef wrapper instead (same for shaders ?)
    pub fn set_material(&mut self, material: &Mat<T>) {
        self.material = material.glob().clone();
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct Model2DGlob {
    group: InstanceGroup2DKey,
}

impl Model2DGlob {
    fn new(group: InstanceGroup2DKey) -> Self {
        Self { group }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceGroup2DKey {
    pub mesh: usize,
    pub camera: usize,
    pub material: usize,
}

impl InstanceGroup2DKey {
    fn new<T>(model: &Model2D<T>) -> Self {
        Self {
            mesh: model.mesh.index(),
            camera: model.camera.index(),
            material: model.material.index(),
        }
    }
}

#[derive(Default, RootNode, NoVisit)]
pub struct InstanceGroups2D {
    pub(crate) groups: FxHashMap<InstanceGroup2DKey, InstanceGroup2D>,
    gpu: GpuHandle,
}

impl Node for InstanceGroups2D {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        for (model_index, model) in ctx.root::<Globals<Model2DGlob>>().get(ctx).deleted_items() {
            self.group_mut(model.group).delete_model(*model_index);
        }
        // TODO: take into account full WGPU reset ?
        let gpu = self.gpu.get(ctx).take();
        for buffer in self.groups.values_mut() {
            buffer.update(gpu.as_deref());
        }
        self.groups.retain(|_, group| !group.buffers.is_empty());
    }
}

impl InstanceGroups2D {
    pub fn group_iter(&self) -> impl Iterator<Item = InstanceGroup2DKey> + '_ {
        self.groups.keys().copied()
    }

    fn register_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let group = InstanceGroup2DKey::new(model);
        self.group_mut(group).register_model(model, data);
    }

    fn update_model<T>(
        &mut self,
        model: &Model2D<T>,
        data: T::InstanceData,
        old_group: InstanceGroup2DKey,
    ) where
        T: Material,
    {
        let group = InstanceGroup2DKey::new(model);
        if group == old_group {
            self.group_mut(group).update_model(model, data);
        } else {
            self.group_mut(old_group).delete_model(model.glob().index());
            self.group_mut(group).register_model(model, data);
        }
    }

    fn group_mut(&mut self, group: InstanceGroup2DKey) -> &mut InstanceGroup2D {
        self.groups.entry(group).or_default()
    }
}

#[derive(Default, Debug)]
pub(crate) struct InstanceGroup2D {
    pub(crate) buffers: FxHashMap<TypeId, InstanceGroupBuffer>,
    pub(crate) model_indexes: Vec<usize>,
    model_positions: FxHashMap<usize, usize>,
}

impl InstanceGroup2D {
    fn register_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let model_index = model.glob().index();
        self.model_positions
            .insert(model_index, self.model_indexes.len());
        self.model_indexes.push(model_index);
        self.buffer_mut::<Instance>()
            .push(bytemuck::cast_slice(&[Instance::new(model)]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T>().push(bytemuck::cast_slice(&[data]));
        }
    }

    fn update_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let position = self.model_positions[&model.glob().index()];
        self.buffer_mut::<Instance>()
            .replace(position, bytemuck::cast_slice(&[Instance::new(model)]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T>()
                .replace(position, bytemuck::cast_slice(&[data]));
        }
    }

    fn delete_model(&mut self, model_index: usize) {
        let position = self
            .model_positions
            .remove(&model_index)
            .expect("internal error: missing model");
        self.model_indexes.swap_remove(position);
        if let Some(&moved_model_index) = self.model_indexes.get(position) {
            self.model_positions.insert(moved_model_index, position);
        }
        for buffer in self.buffers.values_mut() {
            buffer.swap_delete(position);
        }
    }

    fn update(&mut self, gpu: Option<&Gpu>) {
        for buffer in self.buffers.values_mut() {
            buffer.update(gpu);
        }
    }

    fn buffer_mut<T>(&mut self) -> &mut InstanceGroupBuffer
    where
        T: 'static,
    {
        self.buffers
            .entry(TypeId::of::<T>())
            .or_insert_with(|| InstanceGroupBuffer::new::<T>())
    }
}

#[derive(Debug)]
pub(crate) struct InstanceGroupBuffer {
    pub(crate) buffer: Option<Buffer<u8>>,
    pub(crate) data: Vec<u8>,
    item_size: usize,
    is_updated: bool,
}

impl InstanceGroupBuffer {
    fn new<T>() -> Self {
        Self {
            buffer: None,
            data: vec![],
            item_size: mem::size_of::<T>(),
            is_updated: false,
        }
    }

    fn push(&mut self, item: &[u8]) {
        self.data.extend(item);
        self.is_updated = true;
    }

    fn replace(&mut self, position: usize, item: &[u8]) {
        let range = (position * self.item_size)..((position + 1) * self.item_size);
        let old_item = &self.data[range.clone()];
        if old_item != item {
            self.data.splice(range, item.iter().copied());
            self.is_updated = true;
        }
    }

    fn swap_delete(&mut self, position: usize) {
        for i in (0..self.item_size).rev() {
            self.data.swap_remove(position * self.item_size + i);
        }
    }

    fn update(&mut self, gpu: Option<&Gpu>) {
        if let Some(gpu) = gpu {
            if let Some(buffer) = &mut self.buffer {
                if self.is_updated {
                    buffer.update(gpu, &self.data);
                }
            } else {
                self.buffer = Some(Buffer::new(
                    gpu,
                    &self.data,
                    BufferUsages::VERTEX | BufferUsages::COPY_DST,
                    "instance_group",
                ));
            }
        } else {
            self.buffer = None;
        }
        self.is_updated = false;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct Instance {
    transform: [[f32; 4]; 4],
}

impl Instance {
    pub(crate) fn new<T>(model: &Model2D<T>) -> Self {
        let z = (f32::from(model.z_index) + 0.5) / (f32::from(u16::MAX) + 1.) + 0.5;
        Self {
            transform: (Mat4::from_scale(model.size.with_z(0.))
                * Quat::from_z(model.rotation).matrix()
                * Mat4::from_position(model.position.with_z(z)))
            .to_array(),
        }
    }

    pub(crate) fn z(&self) -> f32 {
        self.transform[3][2]
    }
}

impl<const L: u32> VertexBuffer<L> for Instance {
    const ATTRIBUTES: &'static [VertexAttribute] = &vertex_attr_array![
        L => Float32x4,
        L + 1 => Float32x4,
        L + 2 => Float32x4,
        L + 3 => Float32x4,
    ];
    const STEP_MODE: VertexStepMode = VertexStepMode::Instance;
}
