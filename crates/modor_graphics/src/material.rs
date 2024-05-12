use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuHandle, GpuState};
use crate::material::internal::DefaultMaterial2DData;
use crate::model::Model2DGlob;
use crate::texture::TextureProperties;
use crate::{Color, Shader, ShaderGlob, ShaderSource, Size, Texture, TextureGlob, TextureSource};
use bytemuck::Pod;
use modor::{Context, Glob, GlobRef, Node, RootNode, RootNodeHandle, Visit};
use modor_input::modor_math::Vec2;
use modor_resources::Res;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Id, Sampler,
    TextureView,
};

#[derive(Debug, Visit)]
pub struct Mat<T> {
    data: T,
    label: String,
    glob: Glob<Option<MaterialGlob>>,
    gpu: GpuHandle,
    is_invalid: bool,
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

impl<T> Node for Mat<T>
where
    T: Material,
{
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        match self.gpu.get(ctx) {
            GpuState::None => *self.glob.get_mut(ctx) = None,
            GpuState::New(gpu) => self.create_glob(ctx, &gpu),
            GpuState::Same(gpu) => self.update_glob(ctx, &gpu),
        }
    }
}

impl<T> Mat<T>
where
    T: Material,
{
    pub fn new(ctx: &mut Context<'_>, label: impl Into<String>, data: T) -> Self {
        Self {
            data,
            label: label.into(),
            glob: Glob::new(ctx, None),
            gpu: GpuHandle::default(),
            is_invalid: false,
            phantom_data: PhantomData,
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Option<MaterialGlob>> {
        self.glob.as_ref()
    }

    pub fn is_invalid(&self) -> bool {
        self.is_invalid
    }

    fn create_glob(&self, ctx: &mut Context<'_>, gpu: &Gpu) {
        *self.glob.get_mut(ctx) = MaterialGlob::new::<T>(ctx, gpu, &self.data, &self.label);
    }

    fn update_glob(&self, ctx: &mut Context<'_>, gpu: &Gpu) {
        if let Some(mut glob) = self.glob.get_mut(ctx).take() {
            if glob.update(ctx, gpu, &self.data, &self.label).is_some() {
                *self.glob.get_mut(ctx) = Some(glob);
            }
        } else if !self.is_invalid {
            self.create_glob(ctx, gpu);
        }
    }
}

#[derive(Debug)]
pub struct MaterialGlob {
    pub(crate) is_transparent: bool,
    pub(crate) has_instance_data: bool,
    pub(crate) shader: GlobRef<Option<ShaderGlob>>,
    binding_ids: BindingGlobalIds,
    bind_group: BufferBindGroup,
    buffer: Buffer<u8>,
    textures: Vec<GlobRef<Option<TextureGlob>>>,
    data: Vec<u8>,
    gpu_version: u64,
}

impl MaterialGlob {
    pub(crate) fn bind_group(&self, gpu_version: u64) -> Option<&BindGroup> {
        (self.gpu_version == gpu_version).then_some(&self.bind_group.inner)
    }

    fn new<T>(
        ctx: &mut Context<'_>,
        gpu: &Gpu,
        material: &impl Material,
        label: &str,
    ) -> Option<Self>
    where
        T: Material,
    {
        let shader = material.shader(ctx).glob().clone();
        let textures = material.textures(ctx);
        let resource_handle = ctx.root::<DefaultMaterial2DResources>();
        let default_texture = Self::default_texture(ctx, gpu, resource_handle)?;
        let texture_props: Vec<_> = Self::texture_props(ctx, gpu, default_texture, &textures);
        let data = Self::data(material);
        let buffer = Buffer::new(
            gpu,
            &data,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            label,
        );
        let layout = &shader.get(ctx).as_ref()?.material_bind_group_layout;
        Some(Self {
            is_transparent: material.is_transparent()
                || texture_props.iter().any(|texture| texture.is_transparent),
            has_instance_data: mem::size_of::<T::InstanceData>() > 0,
            shader,
            binding_ids: BindingGlobalIds::new(&buffer, &texture_props),
            bind_group: Self::create_bind_group(gpu, &buffer, &texture_props, layout, label),
            buffer,
            textures,
            data,
            gpu_version: gpu.version,
        })
    }

    fn update(
        &mut self,
        ctx: &mut Context<'_>,
        gpu: &Gpu,
        material: &impl Material,
        label: &str,
    ) -> Option<()> {
        self.shader = material.shader(ctx).glob().clone();
        self.textures = material.textures(ctx);
        let data = Self::data(material);
        if self.data != data {
            self.buffer.update(gpu, &data);
            self.data = data;
        }
        let default_texture_handle = ctx.root::<DefaultMaterial2DResources>();
        let default_texture = Self::default_texture(ctx, gpu, default_texture_handle)?;
        let texture_props: Vec<_> = Self::texture_props(ctx, gpu, default_texture, &self.textures);
        self.is_transparent =
            material.is_transparent() || texture_props.iter().any(|texture| texture.is_transparent);
        let binding_ids = BindingGlobalIds::new(&self.buffer, &texture_props);
        if binding_ids != self.binding_ids {
            let layout = &self.shader.get(ctx).as_ref()?.material_bind_group_layout;
            self.bind_group =
                Self::create_bind_group(gpu, &self.buffer, &texture_props, layout, label);
            self.binding_ids = binding_ids;
        }
        Some(())
    }

    fn default_texture<'a>(
        ctx: &'a Context<'_>,
        gpu: &Gpu,
        resources: RootNodeHandle<DefaultMaterial2DResources>,
    ) -> Option<TextureProperties<'a>> {
        resources
            .get(ctx)
            .fallback_texture
            .glob()
            .get(ctx)
            .as_ref()?
            .properties(gpu.version)
    }

    fn texture_props<'a>(
        ctx: &'a Context<'_>,
        gpu: &Gpu,
        default_texture: TextureProperties<'a>,
        textures: &[GlobRef<Option<TextureGlob>>],
    ) -> Vec<TextureProperties<'a>> {
        textures
            .iter()
            .map(|texture| {
                texture
                    .get(ctx)
                    .as_ref()
                    .and_then(|texture| texture.properties(gpu.version))
                    .unwrap_or(default_texture)
            })
            .collect()
    }

    fn data(material: &impl Material) -> Vec<u8> {
        bytemuck::try_cast_slice(&[material.data()])
            .unwrap_or(&[0])
            .into()
    }

    fn create_bind_group(
        gpu: &Gpu,
        buffer: &Buffer<u8>,
        textures: &[TextureProperties<'_>],
        layout: &BindGroupLayout,
        label: &str,
    ) -> BufferBindGroup {
        // TODO: check WGPU errors
        let entries = Self::entries(buffer, textures);
        BufferBindGroup::new(gpu, &entries, layout, label)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn entries<'a>(
        buffer: &'a Buffer<u8>,
        textures: &'a [TextureProperties<'_>],
    ) -> Vec<BindGroupEntry<'a>> {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.resource(),
        }];
        for (i, texture) in textures.iter().enumerate() {
            entries.extend([
                BindGroupEntry {
                    binding: i as u32 * 2 + 1,
                    resource: BindingResource::TextureView(texture.view),
                },
                BindGroupEntry {
                    binding: i as u32 * 2 + 2,
                    resource: BindingResource::Sampler(texture.sampler),
                },
            ]);
        }
        entries
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BindingGlobalIds {
    buffer: Id<wgpu::Buffer>,
    views: Vec<Id<TextureView>>,
    sampler: Vec<Id<Sampler>>,
}

impl BindingGlobalIds {
    fn new(buffer: &Buffer<u8>, textures: &[TextureProperties<'_>]) -> Self {
        Self {
            buffer: buffer.id(),
            views: textures
                .iter()
                .map(|texture| texture.view.global_id())
                .collect(),
            sampler: textures
                .iter()
                .map(|texture| texture.sampler.global_id())
                .collect(),
        }
    }
}

pub trait Material: Sized + 'static {
    type Data: Pod;
    type InstanceData: Pod;

    fn shader<'a>(&self, ctx: &'a mut Context<'_>) -> &'a Shader<Self>;

    fn textures(&self, ctx: &mut Context<'_>) -> Vec<GlobRef<Option<TextureGlob>>>;

    fn is_transparent(&self) -> bool;

    fn data(&self) -> Self::Data;

    fn instance_data(ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData;
}

#[derive(Debug)]
pub struct DefaultMaterial2D {
    pub color: Color,
    pub texture: Option<GlobRef<Option<TextureGlob>>>,
    pub texture_position: Vec2,
    pub texture_size: Vec2,
    pub is_ellipse: bool,
}

impl Default for DefaultMaterial2D {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            texture: None,
            texture_position: Vec2::ZERO,
            texture_size: Vec2::ONE,
            is_ellipse: false,
        }
    }
}

impl Material for DefaultMaterial2D {
    type Data = DefaultMaterial2DData;
    type InstanceData = ();

    fn shader<'a>(&self, ctx: &'a mut Context<'_>) -> &'a Shader<Self> {
        let resources = ctx.root::<DefaultMaterial2DResources>().get(ctx);
        if self.is_ellipse {
            &resources.ellipse_shader
        } else {
            &resources.default_shader
        }
    }

    fn textures(&self, ctx: &mut Context<'_>) -> Vec<GlobRef<Option<TextureGlob>>> {
        let resources = ctx.root::<DefaultMaterial2DResources>().get(ctx);
        vec![self
            .texture
            .clone()
            .unwrap_or_else(|| resources.fallback_texture.glob().clone())]
    }

    fn is_transparent(&self) -> bool {
        self.color.a > 0. && self.color.a < 1.
    }

    fn data(&self) -> Self::Data {
        DefaultMaterial2DData {
            color: self.color.into(),
            texture_part_position: [self.texture_position.x, self.texture_position.y],
            texture_part_size: [self.texture_size.x, self.texture_size.y],
        }
    }

    fn instance_data(_ctx: &mut Context<'_>, _model: &GlobRef<Model2DGlob>) -> Self::InstanceData {}
}

#[derive(Debug, Node, Visit)]
struct DefaultMaterial2DResources {
    default_shader: Res<Shader<DefaultMaterial2D>>,
    ellipse_shader: Res<Shader<DefaultMaterial2D>>,
    fallback_texture: Res<Texture>,
}

impl RootNode for DefaultMaterial2DResources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            default_shader: Res::from_source(
                ctx,
                "default(modor_graphics)",
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/default.wgsl")).into(),
                ),
            ),
            ellipse_shader: Res::from_source(
                ctx,
                "ellipse(modor_graphics)",
                ShaderSource::String(
                    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
                ),
            ),
            fallback_texture: Res::from_source(
                ctx,
                "fallback(modor_graphics)",
                TextureSource::Size(Size::ONE),
            ),
        }
    }
}

mod internal {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
    pub struct DefaultMaterial2DData {
        pub(crate) color: [f32; 4],
        pub(crate) texture_part_position: [f32; 2],
        pub(crate) texture_part_size: [f32; 2],
    }
}
