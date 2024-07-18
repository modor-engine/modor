use crate::buffer::Buffer;
use crate::gpu::Gpu;
use crate::mesh::{MeshGlob, VertexBuffer};
use crate::resources::Resources;
use crate::{
    Camera2DGlob, DestinationModelProps, Material, MaterialGlobRef, Model2DMappings,
    SourceModelProps, Window,
};
use derivative::Derivative;
use fxhash::{FxHashMap, FxHashSet};
use modor::{Builder, Context, Glob, GlobRef, Globals, Node, RootNode, RootNodeHandle, Visit};
use modor_input::modor_math::{Mat4, Quat, Vec2};
use modor_physics::Body2DGlob;
use std::any::TypeId;
use std::marker::PhantomData;
use std::{iter, mem};
use wgpu::{vertex_attr_array, BufferUsages, VertexAttribute, VertexStepMode};

/// The instance of a rendered 2D object.
///
/// Note that in case a material is only used for a specific model, this model can be directly
/// created with a [`Sprite2D`](crate::Sprite2D).
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
///         let material = DefaultMaterial2D::new(ctx)
///             .with_color(color)
///             .with_is_ellipse(true)
///             .into_mat(ctx, "circle");
///         let model = Model2D::new(ctx, material.glob())
///             .with_position(position)
///             .with_size(Vec2::ONE * radius * 2.);
///         Self { material, model }
///     }
/// }
/// ```
#[derive(Derivative, Visit, Builder)]
#[derivative(Debug(bound = ""))]
pub struct Model2D<T> {
    /// The position of the model is world units.
    ///
    /// Default is [`Vec2::ZERO`].
    #[builder(form(value))]
    pub position: Vec2,
    /// The size of the model is world units.
    ///
    /// Default is [`Vec2::ONE`].
    #[builder(form(value))]
    pub size: Vec2,
    /// The rotation of the model in radians.
    ///
    /// Default is `0.0`.
    #[builder(form(value))]
    pub rotation: f32,
    /// The physics body linked to the model.
    ///
    /// At each model update, the position, size and rotation are replaced by those of the body.
    ///
    /// Default is `None`.
    #[builder(form(value))]
    pub body: Option<GlobRef<Body2DGlob>>,
    /// The Z-index of the model.
    ///
    /// [`i16::MIN`] is the farthest from the camera, and [`i16::MAX`] the closest to the camera.
    ///
    /// Default is `0`.
    #[builder(form(value))]
    pub z_index: i16,
    /// The camera on which the model is rendered.
    ///
    /// Default is the default camera of the [`Window`].
    #[builder(form(value))]
    pub camera: GlobRef<Camera2DGlob>,
    /// The material used to render the model.
    #[builder(form(value))]
    pub material: MaterialGlobRef<T>,
    pub(crate) mesh: GlobRef<MeshGlob>,
    glob: Glob<Model2DGlob>,
    groups: RootNodeHandle<InstanceGroups2D>,
    phantom: PhantomData<fn(T)>,
    is_updated: bool,
}

// TODO: what should we do ?
// Model2D::type_ = Model2DType::Static/Model2DType::Dynamic(Option<Glob<Body2DGlob>>)
// Model2D::on_enter does nothing expect position update

impl<T> Node for Model2D<T>
where
    T: Material,
{
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        if self.is_updated {
            return;
        }
        if let Some(body) = &self.body {
            let glob = body.get(ctx);
            self.position = glob.position;
            self.size = glob.size;
            self.rotation = glob.rotation;
        }
        self.is_updated = true;
        let data = T::instance_data(ctx, self.glob());
        let mapping_data = self.mapping_data(ctx);
        self.groups
            .get_mut(ctx)
            .update_model(self, data, mapping_data);
    }
}

impl<T> Model2D<T>
where
    T: Material,
{
    /// Creates a new model.
    pub fn new(ctx: &mut Context<'_>, material: MaterialGlobRef<T>) -> Self {
        let camera = ctx.get_mut::<Window>().camera.glob().clone();
        let mesh = ctx.get_mut::<Resources>().rectangle_mesh.glob().clone();
        let model = Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            body: None,
            z_index: 0,
            glob: Glob::new(ctx, Model2DGlob),
            camera,
            material,
            mesh,
            groups: ctx.handle::<InstanceGroups2D>(),
            phantom: PhantomData,
            is_updated: false,
        };
        let data = T::instance_data(ctx, model.glob());
        let mapping_data = model.mapping_data(ctx);
        model
            .groups
            .get_mut(ctx)
            .register_model(&model, data, mapping_data);
        model
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Model2DGlob> {
        self.glob.as_ref()
    }

    #[allow(clippy::needless_collect)]
    fn mapping_data(&self, ctx: &mut Context<'_>) -> Vec<(DestinationModelProps, Vec<u8>)> {
        let mappings = ctx
            .get_mut::<Model2DMappings>()
            .destinations(SourceModelProps::new(&self.camera, &self.material))
            .collect::<Vec<_>>();
        mappings
            .into_iter()
            .map(|dest| (dest, (dest.generate_instance_data)(ctx, self.glob.index())))
            .collect::<Vec<_>>()
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

    pub(crate) fn register_model<T>(
        &mut self,
        model: &Model2D<T>,
        data: T::InstanceData,
        mapping_data: Vec<(DestinationModelProps, Vec<u8>)>,
    ) where
        T: Material,
    {
        let group = InstanceGroup2DProperties::new(model);
        self.group_mut(group)
            .register_model(model, data, mapping_data);
        let model_index = model.glob().index();
        (self.model_groups.len()..=model_index).for_each(|_| self.model_groups.push(None));
        self.model_groups[model_index] = Some(group);
    }

    pub(crate) fn update_model<T>(
        &mut self,
        model: &Model2D<T>,
        data: T::InstanceData,
        mapping_data: Vec<(DestinationModelProps, Vec<u8>)>,
    ) where
        T: Material,
    {
        let model_index = model.glob().index();
        let old_group =
            self.model_groups[model_index].expect("internal error: missing model groups");
        let group = InstanceGroup2DProperties::new(model);
        if group == old_group {
            self.group_mut(group)
                .update_model(model, data, mapping_data);
        } else {
            self.group_mut(old_group).delete_model(model.glob().index());
            self.group_mut(group)
                .register_model(model, data, mapping_data);
            self.model_groups[model_index] = Some(group);
        }
    }

    fn group_mut(&mut self, group: InstanceGroup2DProperties) -> &mut InstanceGroup2D {
        self.groups.entry(group).or_default()
    }
}

#[derive(Default, Debug)]
pub(crate) struct InstanceGroup2D {
    pub(crate) buffers: FxHashMap<BufferId, InstanceGroupBuffer>,
    pub(crate) model_indexes: Vec<usize>,
    pub(crate) z_indexes: Vec<f32>,
    model_positions: FxHashMap<usize, usize>,
    secondary_type: Option<TypeId>,
    dest_props: FxHashSet<DestinationModelProps>,
}

impl InstanceGroup2D {
    pub(crate) fn primary_buffer(&self) -> Option<&Buffer<u8>> {
        self.buffers[&BufferId::Primary].buffer.as_ref()
    }

    pub(crate) fn secondary_buffer(&self) -> Option<&Buffer<u8>> {
        self.buffers
            .get(&BufferId::Secondary)
            .and_then(|b| b.buffer.as_ref())
    }

    // TODO: render mappings
    // TODO: remove old mappings
    // TODO: explain in doc that not updated models (e.g. in Const<>) will be skipped or have zeroed instance data
    fn register_model<T>(
        &mut self,
        model: &Model2D<T>,
        data: T::InstanceData,
        mapping_data: Vec<(DestinationModelProps, Vec<u8>)>,
    ) where
        T: Material,
    {
        let model_index = model.glob().index();
        let position = self.model_indexes.len();
        self.model_positions.insert(model_index, position);
        self.model_indexes.push(model_index);
        let instance = Instance::new(model);
        self.z_indexes.push(instance.z());
        self.buffer_mut::<T>(BufferId::Primary)
            .push(bytemuck::cast_slice(&[instance]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T>(BufferId::Secondary)
                .push(bytemuck::cast_slice(&[data]));
            self.secondary_type = Some(TypeId::of::<T::InstanceData>());
        }
        for (dest_props, data) in mapping_data {
            if !data.is_empty() {
                self.buffer_mut::<T>(BufferId::Mapping(dest_props))
                    .insert(position, &data);
            }
            self.dest_props.insert(dest_props);
        }
    }

    fn update_model<T>(
        &mut self,
        model: &Model2D<T>,
        data: T::InstanceData,
        mapping_data: Vec<(DestinationModelProps, Vec<u8>)>,
    ) where
        T: Material,
    {
        let position = self.model_positions[&model.glob().index()];
        let instance = Instance::new(model);
        self.z_indexes[position] = instance.z();
        self.buffer_mut::<T>(BufferId::Primary)
            .insert(position, bytemuck::cast_slice(&[instance]));
        if mem::size_of::<T::InstanceData>() > 0 {
            self.buffer_mut::<T>(BufferId::Secondary)
                .insert(position, bytemuck::cast_slice(&[data]));
        }
        for (dest_props, data) in mapping_data {
            if !data.is_empty() {
                self.buffer_mut::<T>(BufferId::Mapping(dest_props))
                    .insert(position, &data);
            }
            self.dest_props.insert(dest_props);
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

    fn buffer_mut<T>(&mut self, id: BufferId) -> &mut InstanceGroupBuffer
    where
        T: Material,
    {
        self.buffers.entry(id).or_insert_with(|| {
            InstanceGroupBuffer::new(match id {
                BufferId::Primary => mem::size_of::<Instance>(),
                BufferId::Secondary => mem::size_of::<T::InstanceData>(),
                BufferId::Mapping(props) => props.instance_data_size,
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BufferId {
    Primary,
    Secondary,
    Mapping(DestinationModelProps),
}

#[derive(Debug)]
pub(crate) struct InstanceGroupBuffer {
    pub(crate) buffer: Option<Buffer<u8>>,
    pub(crate) data: Vec<u8>,
    item_size: usize,
    is_updated: bool,
}

impl InstanceGroupBuffer {
    fn new(item_size: usize) -> Self {
        Self {
            buffer: None,
            data: vec![],
            item_size,
            is_updated: false,
        }
    }

    fn push(&mut self, item: &[u8]) {
        self.data.extend_from_slice(item);
        self.is_updated = true;
    }

    fn insert(&mut self, position: usize, item: &[u8]) {
        if position * self.item_size >= self.data.len() {
            self.data
                .extend(iter::repeat(0).take(position * self.item_size - self.data.len()));
        }
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
