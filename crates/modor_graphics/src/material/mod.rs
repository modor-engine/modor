use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::model::Model2DGlob;
use crate::resources::Resources;
use crate::{DefaultMaterial2D, Shader, ShaderGlobRef, Texture};
use bytemuck::Pod;
use derivative::Derivative;
use log::error;
use modor::{App, FromApp, Glob, GlobRef, Global, StateHandle};
use modor_resources::Res;
use std::any;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use wgpu::{
    BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Id, Sampler, TextureView,
};

/// A material that defines the aspect of a rendered model.
///
/// # Examples
///
/// See [`Model2D`](crate::Model2D).
#[derive(Debug)]
pub struct Mat<T> {
    data: T,
    glob: Glob<MaterialGlob>,
    phantom_data: PhantomData<T>,
}

impl<T> Deref for Mat<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Mat<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Mat<T>
where
    T: Material,
{
    /// Updates the material.
    pub fn update(&mut self, app: &mut App) {
        self.glob
            .take(app, |glob, app| glob.update(app, &self.data));
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> MaterialGlobRef<T> {
        MaterialGlobRef {
            inner: self.glob.to_ref(),
            phantom: PhantomData,
        }
    }
}

/// A trait implemented for types that can be converted to a [`Mat`].
pub trait IntoMat: Sized {
    /// Converts to a [`Mat`].
    fn into_mat(self, app: &mut App) -> Mat<Self>;
}

impl<T> IntoMat for T
where
    T: Material,
{
    fn into_mat(self, app: &mut App) -> Mat<Self> {
        let mut material = Mat {
            data: self,
            glob: Glob::from_app(app),
            phantom_data: PhantomData,
        };
        material.update(app);
        material
    }
}

/// The global data of a [`Mat`] with data of type `T`.
#[derive(Derivative)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct MaterialGlobRef<T> {
    inner: GlobRef<MaterialGlob>,
    phantom: PhantomData<fn(T)>,
}

impl<T> Deref for MaterialGlobRef<T> {
    type Target = GlobRef<MaterialGlob>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// The global data of a [`Mat`].
#[derive(Debug, Global)]
pub struct MaterialGlob {
    pub(crate) is_transparent: bool,
    pub(crate) bind_group: BufferBindGroup,
    pub(crate) binding_ids: BindingGlobalIds,
    pub(crate) shader: GlobRef<Res<Shader>>,
    buffer: MaterialBuffer,
    textures: Vec<GlobRef<Res<Texture>>>,
}

impl FromApp for MaterialGlob {
    fn from_app(app: &mut App) -> Self {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let shader = app.get_mut::<Resources>().empty_shader.deref().to_ref();
        let textures = vec![];
        let resources = app.handle();
        let white_texture = Self::white_texture(app, resources);
        let texture_refs = Self::textures(app, &textures);
        let buffer = MaterialBuffer::new(&gpu);
        let shader_ref = shader.get(app);
        Self {
            is_transparent: false,
            bind_group: Self::create_bind_group::<DefaultMaterial2D>(
                &gpu,
                &buffer,
                &texture_refs,
                white_texture,
                shader_ref,
            ),
            binding_ids: BindingGlobalIds::new(shader_ref, &texture_refs),
            shader,
            buffer,
            textures,
        }
    }
}

impl MaterialGlob {
    fn update<T>(&mut self, app: &mut App, material: &T)
    where
        T: Material,
    {
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        self.shader = material.shader().deref().to_ref();
        self.textures = material.textures();
        self.buffer.update(&gpu, material);
        let resources = app.handle();
        let white_texture = Self::white_texture(app, resources);
        let textures = Self::textures(app, &self.textures);
        self.is_transparent = material.is_transparent()
            || textures.iter().any(|texture| texture.loaded.is_transparent);
        let shader = self.shader.get(app);
        let binding_ids = BindingGlobalIds::new(shader, &textures);
        if binding_ids != self.binding_ids {
            self.bind_group =
                Self::create_bind_group::<T>(&gpu, &self.buffer, &textures, white_texture, shader);
            self.binding_ids = binding_ids;
        }
    }

    fn textures<'a>(app: &'a App, textures: &[GlobRef<Res<Texture>>]) -> Vec<&'a Texture> {
        textures.iter().map(|texture| &**texture.get(app)).collect()
    }

    fn white_texture(app: &App, handle: StateHandle<Resources>) -> &Texture {
        handle.get(app).white_texture.get(app)
    }

    fn create_bind_group<T>(
        gpu: &Gpu,
        buffer: &MaterialBuffer,
        textures: &[&Texture],
        white_texture: &Texture,
        shader: &Shader,
    ) -> BufferBindGroup
    where
        T: Material,
    {
        let entries = Self::create_bind_group_entries::<T>(
            buffer,
            textures,
            white_texture,
            shader.texture_count,
        );
        BufferBindGroup::new(
            gpu,
            &entries,
            &shader.material_bind_group_layout,
            "material",
        )
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_entries<'a, T>(
        buffer: &'a MaterialBuffer,
        textures: &'a [&Texture],
        white_texture: &'a Texture,
        shader_texture_count: u32,
    ) -> Vec<BindGroupEntry<'a>>
    where
        T: Material,
    {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.inner.resource(),
        }];
        for i in 0..shader_texture_count {
            let texture = textures.get(i as usize).unwrap_or_else(|| {
                error!(
                    "Invalid number of textures for material of type `{}`",
                    any::type_name::<T>()
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

    fn update<T>(&mut self, gpu: &Gpu, material: &T)
    where
        T: Material,
    {
        let data = Self::data(material);
        if self.data != data {
            self.inner.update(gpu, &data);
            self.data = data;
        }
    }

    fn data(material: &impl Material) -> Vec<u8> {
        bytemuck::try_cast_slice(&[material.data()])
            .unwrap_or(&[])
            .into()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BindingGlobalIds {
    pub(crate) bind_group_layout: Id<BindGroupLayout>,
    views: Vec<Id<TextureView>>,
    samplers: Vec<Id<Sampler>>,
}

impl BindingGlobalIds {
    fn new(shader: &Shader, textures: &[&Texture]) -> Self {
        Self {
            bind_group_layout: shader.material_bind_group_layout.global_id(),
            views: textures
                .iter()
                .map(|texture| texture.view.global_id())
                .collect(),
            samplers: textures
                .iter()
                .map(|texture| texture.sampler.global_id())
                .collect(),
        }
    }
}

/// A trait for defining [`Mat`] data.
///
/// # Examples
///
/// See code of `custom_shader` example.
pub trait Material: Sized + 'static {
    /// Raw material data type.
    type Data: Pod;
    /// Raw instance data type.
    ///
    /// Each rendered model has its own instance data.
    ///
    /// In case this type has a size of zero with [`mem::size_of`](std::mem::size_of()),
    /// then no instance data are sent to the shader.
    type InstanceData: Pod;

    /// Returns the shader used to make the rendering.
    fn shader(&self) -> ShaderGlobRef<Self>;

    /// Returns the textures sent to the shader.
    fn textures(&self) -> Vec<GlobRef<Res<Texture>>>;

    /// Returns whether the rendered models can be transparent.
    ///
    /// In case `true` is returned, the models will be rendered in Z-index order.
    /// This is less efficient than for opaque models, but this limits the risk of having
    /// rendering artifacts caused by transparency.
    ///
    /// Note that transparency is automatically detected for textures returned by
    /// [`Material::textures`].
    /// It means that if [`Material::is_transparent`]
    /// returns `false` but one of the textures contains transparent pixels, then the models
    /// are considered as transparent.
    fn is_transparent(&self) -> bool;

    /// Returns the raw material data sent to the shader.
    ///
    /// # Platform-specific
    ///
    /// - Web: data size in bytes should be a multiple of 16.
    fn data(&self) -> Self::Data;

    /// Returns the instance data of a given `model`.
    fn instance_data(app: &mut App, model: &Glob<Model2DGlob>) -> Self::InstanceData;
}

pub(crate) mod default_2d;
