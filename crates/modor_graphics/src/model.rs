use crate::buffer::Buffer;
use crate::gpu::Gpu;
use crate::material::InstanceDataType;
use crate::mesh::Mesh;
use crate::mesh::VertexBuffer;
use crate::resources::{Materials, Resources};
use crate::{Camera2DGlob, Mat, Window};
use derivative::Derivative;
use fxhash::FxHashMap;
use modor::{App, Builder, FromApp, Glob, GlobRef, Global, Globals, State, StateHandle};
use modor_input::modor_math::{Mat4, Quat, Vec2};
use modor_physics::Body2D;
use std::any::TypeId;
use std::mem;
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
/// struct Circle {
///     material: MatGlob<DefaultMaterial2D>,
///     model: Model2D,
/// }
///
/// impl Circle {
///     fn new(app: &mut App, position: Vec2, radius: f32, color: Color) -> Self {
///         let material = MatGlob::<DefaultMaterial2D>::from_app(app);
///         DefaultMaterial2DUpdater::default()
///             .color(color)
///             .is_ellipse(true)
///             .apply(app, &material);
///         let model = Model2D::new(app)
///             .with_position(position)
///             .with_size(Vec2::ONE * radius * 2.)
///             .with_material(material.to_ref());
///         Self { material, model }
///     }
///
///     fn update(&mut self, app: &mut App) {
///          self.model.update(app);
///     }
/// }
/// ```
#[derive(Derivative, Builder)]
#[derivative(Debug(bound = ""))]
pub struct Model2D {
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
    pub body: Option<GlobRef<Body2D>>,
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
    pub material: GlobRef<Mat>,
    mesh: GlobRef<Mesh>,
    glob: Glob<Model2DGlob>,
    groups: StateHandle<InstanceGroups2D>,
}

impl Model2D {
    /// Creates a new model.
    pub fn new(app: &mut App) -> Self {
        let camera = app.get_mut::<Window>().camera.glob().to_ref();
        let mesh = app.get_mut::<Resources>().rectangle_mesh.to_ref();
        let material = app.get_mut::<Materials>().default_2d.to_ref();
        let model = Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            body: None,
            z_index: 0,
            glob: Glob::from_app(app),
            camera,
            material,
            mesh,
            groups: app.handle::<InstanceGroups2D>(),
        };
        let data_type = model.material.get(app).instance_data_type;
        let data = (data_type.create_fn)(app, &model.glob);
        model
            .groups
            .get_mut(app)
            .register_model(&model, data, data_type);
        model
    }

    /// Updates the model.
    pub fn update(&mut self, app: &mut App) {
        if let Some(body) = &self.body {
            let glob = body.get(app);
            self.position = glob.position(app);
            self.size = glob.size();
            self.rotation = glob.rotation(app);
        }
        let data_type = self.material.get(app).instance_data_type;
        let data = (data_type.create_fn)(app, &self.glob);
        self.groups.get_mut(app).update_model(self, data, data_type);
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<Model2DGlob> {
        &self.glob
    }
}

/// The global data of a [`Model2D`].
#[non_exhaustive]
#[derive(Default, Debug, Global)]
pub struct Model2DGlob;

/// The properties of an instance group.
///
/// An instance group contains all models that are rendered with the same material, camera and mesh.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InstanceGroup2DProperties {
    /// The index of the [`Mat`](Mat).
    pub material: usize,
    /// The index of the [`Camera2D`](crate::Camera2D).
    pub camera: usize,
    pub(crate) mesh: usize,
}

impl InstanceGroup2DProperties {
    fn new(model: &Model2D) -> Self {
        Self {
            mesh: model.mesh.index(),
            camera: model.camera.index(),
            material: model.material.index(),
        }
    }
}

/// The information about instance groups managed by the graphics crate.
#[derive(FromApp)]
pub struct InstanceGroups2D {
    pub(crate) groups: FxHashMap<InstanceGroup2DProperties, InstanceGroup2D>,
    model_groups: Vec<Option<InstanceGroup2DProperties>>,
}

impl State for InstanceGroups2D {
    fn update(&mut self, app: &mut App) {
        for (model_index, _) in app.get_mut::<Globals<Model2DGlob>>().deleted_items() {
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

    fn register_model(&mut self, model: &Model2D, data: Vec<u8>, data_type: InstanceDataType) {
        let group = InstanceGroup2DProperties::new(model);
        self.group_mut(group).register_model(model, data, data_type);
        let model_index = model.glob.index();
        (self.model_groups.len()..=model_index).for_each(|_| self.model_groups.push(None));
        self.model_groups[model_index] = Some(group);
    }

    fn update_model(&mut self, model: &Model2D, data: Vec<u8>, data_type: InstanceDataType) {
        let model_index = model.glob.index();
        let old_group =
            self.model_groups[model_index].expect("internal error: missing model groups");
        let group = InstanceGroup2DProperties::new(model);
        if group == old_group {
            self.group_mut(group).update_model(model, data, data_type);
        } else {
            self.group_mut(old_group).delete_model(model.glob().index());
            self.group_mut(group).register_model(model, data, data_type);
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

    fn register_model(&mut self, model: &Model2D, data: Vec<u8>, data_type: InstanceDataType) {
        let model_index = model.glob().index();
        self.model_positions
            .insert(model_index, self.model_indexes.len());
        self.model_indexes.push(model_index);
        let instance = Instance::new(model);
        self.z_indexes.push(instance.z());
        self.buffer_mut(TypeId::of::<Instance>(), mem::size_of::<Instance>())
            .push(bytemuck::cast_slice(&[instance]));
        if data_type.size > 0 {
            self.buffer_mut(data_type.type_id, data_type.size)
                .push(&data);
            self.secondary_type = Some(data_type.type_id);
        }
    }

    fn update_model(&mut self, model: &Model2D, data: Vec<u8>, data_type: InstanceDataType) {
        let position = self.model_positions[&model.glob().index()];
        let instance = Instance::new(model);
        self.z_indexes[position] = instance.z();
        self.buffer_mut(TypeId::of::<Instance>(), mem::size_of::<Instance>())
            .replace(position, bytemuck::cast_slice(&[instance]));
        if data_type.size > 0 {
            self.buffer_mut(data_type.type_id, data_type.size)
                .replace(position, bytemuck::cast_slice(&data));
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

    fn buffer_mut(&mut self, type_id: TypeId, type_size: usize) -> &mut InstanceGroupBuffer {
        self.buffers
            .entry(type_id)
            .or_insert_with(|| InstanceGroupBuffer::new(type_size))
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
    fn new(item_size: usize) -> Self {
        Self {
            buffer: None,
            data: vec![],
            item_size,
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
    pub(crate) fn new(model: &Model2D) -> Self {
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
