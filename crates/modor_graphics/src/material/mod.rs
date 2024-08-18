use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::model::Model2DGlob;
use crate::resources::Resources;
use crate::{Shader, Texture};
use bytemuck::Pod;
use derivative::Derivative;
use log::error;
use modor::{App, FromApp, Glob, GlobRef, Global, Globals, State, StateHandle, Update};
use modor_resources::Res;
use std::any::TypeId;
use std::marker::PhantomData;
use std::ops::Deref;
use std::{any, mem};
use wgpu::{BindGroupEntry, BindingResource, BufferUsages};

pub(crate) mod default_2d;

pub use internal::MatUpdater;

/// A [`Mat`] glob.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct MatGlob<T: Material> {
    inner: Glob<Mat>,
    phantom: PhantomData<fn(T)>,
}

impl<T> FromApp for MatGlob<T>
where
    T: Material,
{
    fn from_app(app: &mut App) -> Self {
        let glob = Self {
            inner: Glob::<Mat>::from_app_with(app, |mat, app| {
                mat.take(app, |mat, app| {
                    let data = T::from_app(app);
                    mat.buffer.update(app, data);
                    mat.type_name = any::type_name::<T>();
                    mat.instance_data_type.type_id = TypeId::of::<T::InstanceData>();
                    mat.instance_data_type.size = mem::size_of::<T::InstanceData>();
                    mat.instance_data_type.create_fn =
                        |app, model| bytemuck::cast_slice(&[T::instance_data(app, model)]).to_vec();
                });
            }),
            phantom: PhantomData,
        };
        T::init(app, &glob);
        glob
    }
}

impl<T> Deref for MatGlob<T>
where
    T: Material,
{
    type Target = Glob<Mat>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> MatGlob<T>
where
    T: Material,
{
    /// Retrieves material [`data`](MatUpdater::data).
    pub fn data(&self, app: &App) -> T {
        *bytemuck::from_bytes(&self.get(app).buffer.data)
    }
}

/// A material that defines the aspect of a rendered model.
///
/// # Examples
///
/// See [`Model2D`](crate::Model2D).
#[derive(Debug, Global)]
pub struct Mat {
    pub(crate) is_transparent: bool,
    pub(crate) has_transparent_texture: bool,
    pub(crate) bind_group: BufferBindGroup,
    pub(crate) shader: GlobRef<Res<Shader>>,
    pub(crate) instance_data_type: InstanceDataType,
    buffer: MaterialBuffer,
    textures: Vec<GlobRef<Res<Texture>>>,
    type_name: &'static str,
    resources: StateHandle<Resources>,
}

impl FromApp for Mat {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let shader = app.get_mut::<Resources>().empty_shader.deref().to_ref();
        let textures = vec![];
        let resources = app.handle();
        let white_texture = Self::white_texture(app, resources);
        let texture_refs = Self::retrieve_textures(app, &textures);
        let buffer = MaterialBuffer::new(&gpu);
        let shader_ref = shader.get(app);
        Self {
            is_transparent: false,
            has_transparent_texture: false,
            bind_group: Self::create_bind_group(
                &gpu,
                &buffer,
                &texture_refs,
                white_texture,
                shader_ref,
                "",
            ),
            shader,
            instance_data_type: InstanceDataType {
                type_id: TypeId::of::<()>(),
                size: 0,
                create_fn: |_, _| panic!("material not created with `MatGlob`"),
            },
            buffer,
            textures,
            type_name: "",
            resources,
        }
    }
}

impl<T> MatUpdater<'_, T>
where
    T: Material,
{
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &MatGlob<T>) {
        let data = self.data.take_value(|| glob.data(app));
        glob.take(app, |mat, app| {
            Update::apply(&mut self.is_transparent, &mut mat.is_transparent);
            if let Some(data) = data {
                mat.buffer.update(app, data);
            }
            let mut is_bind_group_changed = false;
            if let Some(shader) = self.shader.take_value(|| unreachable!()) {
                is_bind_group_changed = shader.index() != mat.shader.index();
                mat.shader = shader.deref().to_ref();
            }
            if let Some(textures) = self.textures.take_value_checked(|| mat.textures.clone()) {
                is_bind_group_changed = true;
                mat.textures = textures;
                mat.has_transparent_texture = Mat::retrieve_textures(app, &mat.textures)
                    .iter()
                    .any(|texture| texture.loaded.is_transparent);
            }
            if is_bind_group_changed {
                mat.refresh_bind_group(app);
            }
        });
    }
}

impl Mat {
    /// Retrieves material [`shader`](MatUpdater::shader).
    pub fn shader(&self) -> &GlobRef<Res<Shader>> {
        &self.shader
    }

    /// Retrieves material [`textures`](MatUpdater::textures).
    pub fn textures(&self) -> impl Iterator<Item = &GlobRef<Res<Texture>>> {
        self.textures.iter()
    }

    fn refresh_bind_group(&mut self, app: &mut App) {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let shader = self.shader.get(app);
        let white_texture = self.resources.get(app).white_texture.get(app);
        let textures = Self::retrieve_textures(app, &self.textures);
        self.bind_group = Self::create_bind_group(
            &gpu,
            &self.buffer,
            &textures,
            white_texture,
            shader,
            self.type_name,
        );
    }

    fn retrieve_textures<'a>(app: &'a App, textures: &[GlobRef<Res<Texture>>]) -> Vec<&'a Texture> {
        textures.iter().map(|texture| &**texture.get(app)).collect()
    }

    fn white_texture(app: &App, handle: StateHandle<Resources>) -> &Texture {
        handle.get(app).white_texture.get(app)
    }

    fn create_bind_group(
        gpu: &Gpu,
        buffer: &MaterialBuffer,
        textures: &[&Texture],
        white_texture: &Texture,
        shader: &Shader,
        material_type_name: &str,
    ) -> BufferBindGroup {
        let entries = Self::create_bind_group_entries(
            buffer,
            textures,
            white_texture,
            shader.texture_count,
            material_type_name,
        );
        BufferBindGroup::new(
            gpu,
            &entries,
            &shader.material_bind_group_layout,
            "material",
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_entries<'a>(
        buffer: &'a MaterialBuffer,
        textures: &'a [&Texture],
        white_texture: &'a Texture,
        shader_texture_count: u32,
        material_type_name: &str,
    ) -> Vec<BindGroupEntry<'a>> {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.inner.resource(),
        }];
        for i in 0..shader_texture_count {
            let texture = textures.get(i as usize).unwrap_or_else(|| {
                error!(
                    "Invalid number of textures for material of type `{}`",
                    material_type_name
                );
                &white_texture
            });
            entries.extend([
                BindGroupEntry {
                    binding: i * 2 + 1,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: i * 2 + 2,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ]);
        }
        entries
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct InstanceDataType {
    pub(crate) type_id: TypeId,
    pub(crate) size: usize,
    pub(crate) create_fn: fn(&mut App, &Glob<Model2DGlob>) -> Vec<u8>,
}

#[derive(Debug)]
struct MaterialBuffer {
    inner: Buffer<u8>,
    data: Vec<u8>,
}

impl MaterialBuffer {
    fn new(gpu: &Gpu) -> Self {
        Self {
            inner: Buffer::new(
                gpu,
                &[],
                BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                "material",
            ),
            data: vec![],
        }
    }

    fn update<T>(&mut self, app: &mut App, material: T)
    where
        T: Material,
    {
        let data = bytemuck::try_cast_slice(&[material])
            .unwrap_or(&[])
            .to_vec();
        if self.data != data {
            let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
            self.inner.update(&gpu, &data);
            self.data = data;
        }
    }
}

/// A trait for defining [`Mat`] data that are sent to a shader.
///
/// # Platform-specific
///
/// - Web: material type size in bytes should be a multiple of 16.
///
/// # Examples
///
/// See code of `custom_shader` example.
pub trait Material: FromApp + Pod + Sized + 'static {
    /// Raw instance data type.
    ///
    /// Each rendered model has its own instance data.
    ///
    /// In case this type has a size of zero with [`mem::size_of`](mem::size_of()),
    /// then no instance data are sent to the shader.
    type InstanceData: Pod;

    /// Initializes the material.
    fn init(app: &mut App, glob: &MatGlob<Self>);

    /// Returns the instance data of a given `model`.
    fn instance_data(app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData;
}

#[derive(Debug, FromApp, State)]
pub(crate) struct MaterialManager {
    loaded_shader_indexes: Vec<usize>,
    loaded_texture_indexes: Vec<usize>,
}

impl MaterialManager {
    pub(crate) fn register_loaded_shader(&mut self, index: usize) {
        self.loaded_shader_indexes.push(index);
    }

    pub(crate) fn register_loaded_texture(&mut self, index: usize) {
        self.loaded_texture_indexes.push(index);
    }

    pub(crate) fn update_material_bind_groups(&mut self, app: &mut App) {
        app.take::<Globals<Mat>, _>(|materials, app| {
            for shader_index in self.loaded_shader_indexes.drain(..) {
                for mat in materials.iter_mut() {
                    if mat.shader.index() == shader_index {
                        mat.refresh_bind_group(app);
                    }
                }
            }
            for texture_index in self.loaded_texture_indexes.drain(..) {
                for mat in materials.iter_mut() {
                    if mat
                        .textures
                        .iter()
                        .any(|texture| texture.index() == texture_index)
                    {
                        mat.refresh_bind_group(app);
                    }
                }
            }
        });
    }
}

mod internal {
    use crate::{ShaderGlobRef, Texture};
    use modor::{GlobRef, Updater};
    use modor_resources::Res;

    // this type is only used to generate `MatUpdater`
    #[derive(Updater)]
    #[allow(dead_code, unreachable_pub)]
    pub struct Mat<T> {
        /// Material data sent to the shader.
        #[updater(field, for_field)]
        pub(super) data: T,
        /// Whether the rendered models can be transparent.
        ///
        /// If `true`, the models will be rendered in Z-index order.
        /// This is less efficient than for opaque models, but this limits the risk of having
        /// rendering artifacts caused by transparency.
        ///
        /// If [`is_transparent`](MatUpdater::is_transparent)
        /// is `false` but one of the [`textures`](MatUpdater::textures) contains transparent
        /// pixels, then the models are considered as transparent.
        ///
        /// Default is `false`.
        #[updater(field, for_field)]
        pub(super) is_transparent: bool,
        /// Shader used to make the rendering.
        ///
        /// Default is a shader that doesn't render anything.
        #[updater(field)]
        pub(super) shader: ShaderGlobRef<T>,
        /// Textures sent to the shader.
        ///
        /// Default is no texture.
        #[updater(field, for_field)]
        pub(super) textures: Vec<GlobRef<Res<Texture>>>,
    }
}
