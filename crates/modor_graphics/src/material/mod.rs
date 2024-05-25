use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::model::Model2DGlob;
use crate::resources::GraphicsResources;
use crate::shader::glob::ShaderGlob;
use crate::texture::glob::TextureGlob;
use crate::ShaderGlobRef;
use bytemuck::Pod;
use derivative::Derivative;
use log::error;
use modor::{Context, Glob, GlobRef, Node, RootNodeHandle, Visit};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use wgpu::{
    BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Id, Sampler, TextureView,
};

/// A material that defines the aspect of a rendered model.
///
/// # Examples
///
/// See [`Model2D`](crate::Model2D).
#[derive(Debug, Visit)]
pub struct Mat<T: Node> {
    data: T,
    label: String,
    glob: Glob<MaterialGlob>,
    updated_glob: MaterialGlob, // used to update the glob without borrowing Context
    phantom_data: PhantomData<T>,
}

impl<T> Deref for Mat<T>
where
    T: Node,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Mat<T>
where
    T: Node,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> Node for Mat<T>
where
    T: Material,
{
    fn on_exit(&mut self, ctx: &mut Context<'_>) {
        mem::swap(self.glob.get_mut(ctx), &mut self.updated_glob);
        self.updated_glob.update(ctx, &self.data, &self.label);
        mem::swap(self.glob.get_mut(ctx), &mut self.updated_glob);
    }
}

impl<T> Mat<T>
where
    T: Material,
{
    /// Creates a new material.
    ///
    /// The `label` is used to identity the material in logs.
    pub fn new(ctx: &mut Context<'_>, label: impl Into<String>, data: T) -> Self {
        let label = label.into();
        let glob = MaterialGlob::new(ctx, &data, &label);
        let dummy_glob = MaterialGlob::new(ctx, &data, &label);
        Self {
            data,
            label,
            glob: Glob::new(ctx, glob),
            updated_glob: dummy_glob,
            phantom_data: PhantomData,
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> MaterialGlobRef<T> {
        MaterialGlobRef {
            inner: self.glob.as_ref().clone(),
            phantom: PhantomData,
        }
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
#[derive(Debug)]
pub struct MaterialGlob {
    pub(crate) is_transparent: bool,
    pub(crate) bind_group: BufferBindGroup,
    pub(crate) binding_ids: BindingGlobalIds,
    pub(crate) shader: GlobRef<ShaderGlob>,
    buffer: MaterialBuffer,
    textures: Vec<GlobRef<TextureGlob>>,
}

impl MaterialGlob {
    fn new<T>(ctx: &mut Context<'_>, material: &T, label: &str) -> Self
    where
        T: Material,
    {
        let gpu = ctx.get_mut::<GpuManager>().get_or_init().clone();
        let shader = material.shader().deref().clone();
        let textures = material.textures();
        let resources = ctx.handle();
        let white_texture = Self::white_texture(ctx, resources);
        let texture_refs = Self::textures(ctx, &textures);
        let buffer = MaterialBuffer::new(&gpu, material, label);
        let shader_ref = shader.get(ctx);
        Self {
            is_transparent: material.is_transparent()
                || texture_refs.iter().any(|texture| texture.is_transparent),
            bind_group: Self::create_bind_group(
                &gpu,
                &buffer,
                &texture_refs,
                white_texture,
                shader_ref,
                label,
            ),
            binding_ids: BindingGlobalIds::new(shader_ref, &texture_refs),
            shader,
            buffer,
            textures,
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>, material: &impl Material, label: &str) {
        let gpu = ctx.get_mut::<GpuManager>().get_or_init().clone();
        self.shader = material.shader().deref().clone();
        self.textures = material.textures();
        self.buffer.update(&gpu, material);
        let resources = ctx.handle();
        let white_texture = Self::white_texture(ctx, resources);
        let textures = Self::textures(ctx, &self.textures);
        self.is_transparent =
            material.is_transparent() || textures.iter().any(|texture| texture.is_transparent);
        let shader = self.shader.get(ctx);
        let binding_ids = BindingGlobalIds::new(shader, &textures);
        if binding_ids != self.binding_ids {
            self.bind_group = Self::create_bind_group(
                &gpu,
                &self.buffer,
                &textures,
                white_texture,
                shader,
                label,
            );
            self.binding_ids = binding_ids;
        }
    }

    fn textures<'a>(
        ctx: &'a Context<'_>,
        textures: &[GlobRef<TextureGlob>],
    ) -> Vec<&'a TextureGlob> {
        textures.iter().map(|texture| texture.get(ctx)).collect()
    }

    fn white_texture<'a>(
        ctx: &'a Context<'_>,
        handle: RootNodeHandle<GraphicsResources>,
    ) -> &'a TextureGlob {
        handle.get(ctx).white_texture.glob().get(ctx)
    }

    fn create_bind_group(
        gpu: &Gpu,
        buffer: &MaterialBuffer,
        textures: &[&TextureGlob],
        white_texture: &TextureGlob,
        shader: &ShaderGlob,
        label: &str,
    ) -> BufferBindGroup {
        let entries = Self::create_bind_group_entries(
            buffer,
            textures,
            white_texture,
            shader.texture_count,
            label,
        );
        BufferBindGroup::new(gpu, &entries, &shader.material_bind_group_layout, label)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn create_bind_group_entries<'a>(
        buffer: &'a MaterialBuffer,
        textures: &'a [&TextureGlob],
        white_texture: &'a TextureGlob,
        shader_texture_count: u32,
        label: &str,
    ) -> Vec<BindGroupEntry<'a>> {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.inner.resource(),
        }];
        for i in 0..shader_texture_count {
            let texture = textures.get(i as usize).unwrap_or_else(|| {
                error!("Invalid number of textures for material `{}`", label);
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
    fn new<T>(gpu: &Gpu, material: &T, label: &str) -> Self
    where
        T: Material,
    {
        let data = Self::data(material);
        Self {
            inner: Buffer::new(
                gpu,
                &data,
                BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                label,
            ),
            data,
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
            .unwrap_or(&[0])
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
    fn new(shader: &ShaderGlob, textures: &[&TextureGlob]) -> Self {
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
pub trait Material: Sized + Node + 'static {
    /// Raw material data type.
    type Data: Pod;
    /// Raw instance data type.
    ///
    /// Each rendered model has its own instance data.
    ///
    /// In case this type has a size of zero with [`mem::size_of`](mem::size_of()),
    /// then no instance data are sent to the shader.
    type InstanceData: Pod;

    /// Returns the shader used to make the rendering.
    fn shader(&self) -> ShaderGlobRef<Self>;

    /// Returns the textures sent to the shader.
    fn textures(&self) -> Vec<GlobRef<TextureGlob>>;

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
    fn instance_data(ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData;
}

pub(crate) mod default_2d;
