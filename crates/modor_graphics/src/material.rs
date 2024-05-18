use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::material::internal::DefaultMaterial2DData;
use crate::model::Model2DGlob;
use crate::{Color, Shader, ShaderGlob, ShaderSource, Size, Texture, TextureGlob, TextureSource};
use bytemuck::Pod;
use modor::{Context, Glob, GlobRef, Node, RootNode, Visit};
use modor_input::modor_math::Vec2;
use modor_resources::Res;
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use wgpu::{
    BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Id, Sampler, TextureView,
};

#[derive(Debug, Visit)]
pub struct Mat<T> {
    data: T,
    label: String,
    glob: Glob<Option<MaterialGlob>>, // TODO: remove Option by derive Node for the glob
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
        if let Some(mut glob) = self.glob.get_mut(ctx).take() {
            glob.update(ctx, &self.data, &self.label);
            *self.glob.get_mut(ctx) = Some(glob);
        }
    }
}

impl<T> Mat<T>
where
    T: Material,
{
    pub fn new(ctx: &mut Context<'_>, label: impl Into<String>, data: T) -> Self {
        let label = label.into();
        let glob = MaterialGlob::new(ctx, &data, &label);
        Self {
            data,
            label,
            glob: Glob::new(ctx, Some(glob)),
            phantom_data: PhantomData,
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &GlobRef<Option<MaterialGlob>> {
        self.glob.as_ref()
    }
}

#[derive(Debug)]
pub struct MaterialGlob {
    pub(crate) is_transparent: bool,
    pub(crate) has_instance_data: bool,
    pub(crate) bind_group: BufferBindGroup,
    pub(crate) binding_ids: BindingGlobalIds,
    pub(crate) shader: GlobRef<ShaderGlob>,
    buffer: Buffer<u8>,
    textures: Vec<GlobRef<TextureGlob>>,
    data: Vec<u8>,
}

impl MaterialGlob {
    fn new<T>(ctx: &mut Context<'_>, material: &T, label: &str) -> Self
    where
        T: Material,
    {
        let gpu = ctx.root::<GpuManager>().get_mut(ctx).get().clone();
        let shader = material.shader(ctx).glob().clone();
        let textures = material.textures(ctx);
        let texture_refs: Vec<_> = Self::texture_refs(ctx, &textures);
        let data = Self::data(material);
        let buffer = Buffer::new(
            &gpu,
            &data,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            label,
        );
        let shader_ref = shader.get(ctx);
        Self {
            is_transparent: material.is_transparent()
                || texture_refs.iter().any(|texture| texture.is_transparent),
            has_instance_data: mem::size_of::<T::InstanceData>() > 0,
            bind_group: Self::create_bind_group(&gpu, &buffer, &texture_refs, shader_ref, label),
            binding_ids: BindingGlobalIds::new(shader_ref, &texture_refs),
            shader,
            buffer,
            textures,
            data,
        }
    }

    fn update(&mut self, ctx: &mut Context<'_>, material: &impl Material, label: &str) {
        let gpu = ctx.root::<GpuManager>().get_mut(ctx).get().clone();
        self.shader = material.shader(ctx).glob().clone();
        self.textures = material.textures(ctx);
        let data = Self::data(material);
        if self.data != data {
            self.buffer.update(&gpu, &data);
            self.data = data;
        }
        let texture_refs: Vec<_> = Self::texture_refs(ctx, &self.textures);
        self.is_transparent =
            material.is_transparent() || texture_refs.iter().any(|texture| texture.is_transparent);
        let shader = self.shader.get(ctx);
        let binding_ids = BindingGlobalIds::new(shader, &texture_refs);
        if binding_ids != self.binding_ids {
            self.bind_group =
                Self::create_bind_group(&gpu, &self.buffer, &texture_refs, shader, label);
            self.binding_ids = binding_ids;
        }
    }

    fn texture_refs<'a>(
        ctx: &'a Context<'_>,
        textures: &[GlobRef<TextureGlob>],
    ) -> Vec<&'a TextureGlob> {
        textures.iter().map(|texture| texture.get(ctx)).collect()
    }

    fn data(material: &impl Material) -> Vec<u8> {
        bytemuck::try_cast_slice(&[material.data()])
            .unwrap_or(&[0])
            .into()
    }

    fn create_bind_group(
        gpu: &Gpu,
        buffer: &Buffer<u8>,
        textures: &[&TextureGlob],
        shader: &ShaderGlob,
        label: &str,
    ) -> BufferBindGroup {
        // TODO: check WGPU errors -> or adapt number of texture depending on shader + log error ?
        // TODO: handle case where texture_count mismatch during rendering
        let entries = Self::entries(buffer, textures, shader.texture_count);
        BufferBindGroup::new(gpu, &entries, &shader.material_bind_group_layout, label)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn entries<'a>(
        buffer: &'a Buffer<u8>,
        textures: &'a [&TextureGlob],
        shader_texture_count: u32,
    ) -> Vec<BindGroupEntry<'a>> {
        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: buffer.resource(),
        }];
        for (i, texture) in textures.iter().enumerate() {
            if i >= shader_texture_count as usize {
                break;
            }
            entries.extend([
                BindGroupEntry {
                    binding: i as u32 * 2 + 1,
                    resource: BindingResource::TextureView(&texture.view),
                },
                BindGroupEntry {
                    binding: i as u32 * 2 + 2,
                    resource: BindingResource::Sampler(&texture.sampler),
                },
            ]);
        }
        entries
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

pub trait Material: Sized + 'static {
    type Data: Pod;
    type InstanceData: Pod;

    fn shader<'a>(&self, ctx: &'a mut Context<'_>) -> &'a Res<Shader<Self>>;

    fn textures(&self, ctx: &mut Context<'_>) -> Vec<GlobRef<TextureGlob>>;

    fn is_transparent(&self) -> bool;

    fn data(&self) -> Self::Data;

    fn instance_data(ctx: &mut Context<'_>, model: &GlobRef<Model2DGlob>) -> Self::InstanceData;
}

#[derive(Debug)]
pub struct DefaultMaterial2D {
    pub color: Color,
    pub texture: Option<GlobRef<TextureGlob>>,
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

    fn shader<'a>(&self, ctx: &'a mut Context<'_>) -> &'a Res<Shader<Self>> {
        let resources = ctx.root::<DefaultMaterial2DResources>().get(ctx);
        if self.is_ellipse {
            &resources.ellipse_shader
        } else {
            &resources.default_shader
        }
    }

    fn textures(&self, ctx: &mut Context<'_>) -> Vec<GlobRef<TextureGlob>> {
        let resources = ctx.root::<DefaultMaterial2DResources>().get(ctx);
        vec![self
            .texture
            .clone()
            .unwrap_or_else(|| resources.white_texture.glob().clone())]
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
    white_texture: Res<Texture>,
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
            // TODO: restore
            // ellipse_shader: Res::from_source(
            //     ctx,
            //     "ellipse(modor_graphics)",
            //     ShaderSource::String(
            //         include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/ellipse.wgsl")).into(),
            //     ),
            // ),
            ellipse_shader: Res::from_path(
                ctx,
                "ellipse(modor_graphics)",
                "../../crates/modor_graphics/res/ellipse.wgsl",
            ),
            white_texture: Res::from_source(
                ctx,
                "white(modor_graphics)",
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
