use crate::buffer::Buffer;
use crate::gpu::Gpu;
use crate::mesh::MeshGlob;
use crate::mesh::VertexBuffer;
use crate::resources::GraphicsResources;
use crate::{Camera2DGlob, Material, MaterialGlobRef, Window};
use derivative::Derivative;
use fxhash::FxHashMap;
use modor::{Context, Glob, GlobRef, Globals, Node, RootNode, RootNodeHandle, Visit};
use modor_input::modor_math::{Mat4, Quat, Vec2};
use std::any::TypeId;
use std::marker::PhantomData;
use std::mem;
use wgpu::{vertex_attr_array, BufferUsages, VertexAttribute, VertexStepMode};

/// The instance of a rendered object.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_physics::modor_math::*;
/// #
/// #[derive(Node, Visit)]
/// struct Circle {
///     material: Mat<DefaultMaterial2D>,
///     model: Model2D<DefaultMaterial2D>,
/// }
///
/// impl Circle {
///     fn new(ctx: &mut Context<'_>, position: Vec2, radius: f32, color: Color) -> Self {
///         let mut material_data = DefaultMaterial2D::new(ctx);
///         material_data.color = color;
///         material_data.is_ellipse = true;
///         let material = Mat::new(ctx, "circle", material_data);
///         let mut model = Model2D::new(ctx, material.glob());
///         model.position = position;
///         model.size = Vec2::ONE * radius * 2.;
///         Self { material, model }
///     }
/// }
/// ```
#[derive(Derivative, Visit)]
#[derivative(Debug(bound = ""))]
pub struct Model2D<T> {
    /// The position of the model is world units.
    ///
    /// Default is [`Vec2::ZERO`].
    pub position: Vec2,
    /// The size of the model is world units.
    ///
    /// Default is [`Vec2::ONE`].
    pub size: Vec2,
    /// The rotation of the model in radians.
    ///
    /// Default is `0.0`.
    pub rotation: f32,
    /// The Z-index of the model.
    ///
    /// [`i16::MIN`] is the farthest from the camera, and [`i16::MAX`] the closest to the camera.
    ///
    /// Default is `0`.
    pub z_index: i16,
    /// The camera on which the model is rendered.
    ///
    /// Default is the default camera of the [`Window`].
    pub camera: GlobRef<Camera2DGlob>,
    /// The material used to render the model.
    pub material: MaterialGlobRef<T>,
    mesh: GlobRef<MeshGlob>,
    glob: Glob<Model2DGlob>,
    groups: RootNodeHandle<InstanceGroups2D>,
    phantom: PhantomData<fn(T)>,
}

impl<T> Node for Model2D<T>
where
    T: Material,
{
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let data = T::instance_data(ctx, self.glob());
        self.groups.get_mut(ctx).update_model(self, data);
    }
}

impl<T> Model2D<T>
where
    T: Material,
{
    /// Creates a new model.
    pub fn new(ctx: &mut Context<'_>, material: MaterialGlobRef<T>) -> Self {
        let camera = ctx.get_mut::<Window>().camera.glob().clone();
        let mesh = ctx
            .get_mut::<GraphicsResources>()
            .rectangle_mesh
            .glob()
            .clone();
        let model = Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            z_index: 0,
            glob: Glob::new(ctx, Model2DGlob),
            camera,
            material,
            mesh,
            groups: ctx.handle::<InstanceGroups2D>(),
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
}

/// The global data of a [`Model2D`].
#[non_exhaustive]
#[derive(Debug)]
pub struct Model2DGlob;

/// The properties of an instance group.
///
/// An instance group contains all models that are rendered with the same material, camera and mesh.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceGroup2DProperties {
    /// The index of the [`Mat`](crate::Mat) global data.
    pub material: usize,
    /// The index of the [`Camera2D`](crate::Camera2D) global data.
    pub camera: usize,
    pub(crate) mesh: usize,
}

impl InstanceGroup2DProperties {
    fn new<T>(model: &Model2D<T>) -> Self {
        Self {
            mesh: model.mesh.index(),
            camera: model.camera.index(),
            material: model.material.index(),
        }
    }
}

/// The information about instance groups managed by the graphics crate.
#[derive(Default, RootNode, Visit)]
pub struct InstanceGroups2D {
    pub(crate) groups: FxHashMap<InstanceGroup2DProperties, InstanceGroup2D>,
    model_groups: Vec<Option<InstanceGroup2DProperties>>,
}

impl Node for InstanceGroups2D {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        for (model_index, _) in ctx.get_mut::<Globals<Model2DGlob>>().deleted_items() {
            let group = self.model_groups[*model_index]
                .take()
                .expect("internal error: missing model groups");
            self.group_mut(group).delete_model(*model_index);
        }
        self.groups.retain(|_, group| !group.buffers.is_empty());
    }
}

impl InstanceGroups2D {
    /// Returns an iterator on all existing instance groups.
    pub fn group_iter(&self) -> impl Iterator<Item = InstanceGroup2DProperties> + '_ {
        self.groups.keys().copied()
    }

    pub(crate) fn sync(&mut self, gpu: &Gpu) {
        for group in self.groups.values_mut() {
            group.sync(gpu);
        }
    }

    fn register_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let group = InstanceGroup2DProperties::new(model);
        self.group_mut(group).register_model(model, data);
        let model_index = model.glob.index();
        (self.model_groups.len()..=model_index).for_each(|_| self.model_groups.push(None));
        self.model_groups[model_index] = Some(group);
    }

    fn update_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let model_index = model.glob.index();
        let old_group =
            self.model_groups[model_index].expect("internal error: missing model groups");
        let group = InstanceGroup2DProperties::new(model);
        if group == old_group {
            self.group_mut(group).update_model(model, data);
        } else {
            self.group_mut(old_group).delete_model(model.glob().index());
            self.group_mut(group).register_model(model, data);
            self.model_groups[model_index] = Some(group);
        }
    }

    fn group_mut(&mut self, group: InstanceGroup2DProperties) -> &mut InstanceGroup2D {
        self.groups.entry(group).or_default()
    }
}

#[derive(Default, Debug)]
pub(crate) struct InstanceGroup2D {
    pub(crate) buffers: FxHashMap<TypeId, InstanceGroupBuffer>,
    pub(crate) model_indexes: Vec<usize>,
    pub(crate) z_indexes: Vec<f32>,
    model_positions: FxHashMap<usize, usize>,
    secondary_type: Option<TypeId>,
}

impl InstanceGroup2D {
    pub(crate) fn primary_buffer(&self) -> Option<&Buffer<u8>> {
        self.buffers[&TypeId::of::<Instance>()].buffer.as_ref()
    }

    pub(crate) fn secondary_buffer(&self) -> Option<&Buffer<u8>> {
        self.secondary_type
            .and_then(|type_id| self.buffers[&type_id].buffer.as_ref())
    }

    fn register_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let model_index = model.glob().index();
        self.model_positions
            .insert(model_index, self.model_indexes.len());
        self.model_indexes.push(model_index);
        let instance = Instance::new(model);
        self.z_indexes.push(instance.z());
        self.buffer_mut::<Instance>()
            .push(bytemuck::cast_slice(&[instance]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T::InstanceData>()
                .push(bytemuck::cast_slice(&[data]));
            self.secondary_type = Some(TypeId::of::<T::InstanceData>());
        }
    }

    fn update_model<T>(&mut self, model: &Model2D<T>, data: T::InstanceData)
    where
        T: Material,
    {
        let position = self.model_positions[&model.glob().index()];
        let instance = Instance::new(model);
        self.z_indexes[position] = instance.z();
        self.buffer_mut::<Instance>()
            .replace(position, bytemuck::cast_slice(&[instance]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T::InstanceData>()
                .replace(position, bytemuck::cast_slice(&[data]));
        }
    }

    fn delete_model(&mut self, model_index: usize) {
        let position = self
            .model_positions
            .remove(&model_index)
            .expect("internal error: missing model");
        self.model_indexes.swap_remove(position);
        self.z_indexes.swap_remove(position);
        if let Some(&moved_model_index) = self.model_indexes.get(position) {
            self.model_positions.insert(moved_model_index, position);
        }
        for buffer in self.buffers.values_mut() {
            buffer.swap_delete(position);
        }
    }

    fn sync(&mut self, gpu: &Gpu) {
        for buffer in self.buffers.values_mut() {
            buffer.sync(gpu);
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
        let buffer_range = (position * self.item_size)..((position + 1) * self.item_size);
        if &self.data[buffer_range.clone()] != item {
            for (item_byte, buffer_byte) in buffer_range.enumerate() {
                self.data[buffer_byte] = item[item_byte];
            }
            self.is_updated = true;
        }
    }

    fn swap_delete(&mut self, position: usize) {
        for i in (0..self.item_size).rev() {
            self.data.swap_remove(position * self.item_size + i);
        }
    }

    fn sync(&mut self, gpu: &Gpu) {
        if self.is_updated {
            self.buffer
                .get_or_insert_with(|| {
                    Buffer::new(
                        gpu,
                        &self.data,
                        BufferUsages::VERTEX | BufferUsages::COPY_DST,
                        "instance_group",
                    )
                })
                .update(gpu, &self.data);
            self.is_updated = false;
        }
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
