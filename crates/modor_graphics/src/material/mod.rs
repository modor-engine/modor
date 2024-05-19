use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::model::Model2DGlob;
use crate::{GraphicsResources, ShaderGlob, ShaderGlobRef, TextureGlob};
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
        let gpu = ctx.get_mut::<GpuManager>().get().clone();
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
        let gpu = ctx.get_mut::<GpuManager>().get().clone();
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

pub trait Material: Sized + Node + 'static {
    type Data: Pod;
    type InstanceData: Pod;

    fn shader(&self) -> ShaderGlobRef<Self>;

    fn textures(&self) -> Vec<GlobRef<TextureGlob>>;

    fn is_transparent(&self) -> bool;

    fn data(&self) -> Self::Data;

    fn instance_data(ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData;
}

pub(crate) mod default_2d;
