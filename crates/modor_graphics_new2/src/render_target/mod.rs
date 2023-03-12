use crate::data::size::NonZeroSize;
use crate::gpu_data::buffer::DynamicBuffer;
use crate::instances::opaque::OpaqueInstanceRegistry;
use crate::instances::transparent::TransparentInstanceRegistry;
use crate::instances::{GroupKey, Instance};
use crate::render_target::window::WindowTarget;
use crate::resources::camera::Camera2DRegistry;
use crate::resources::material::MaterialRegistry;
use crate::resources::mesh::{Mesh, MeshRegistry};
use crate::resources::shader::{Shader, ShaderRegistry};
use crate::resources::texture::{TextureKey, TextureRegistry};
use crate::{
    Camera2D, Color, GraphicsModule, IntoResourceKey, Material, Resource, ResourceKey,
    ResourceRegistry, ResourceState, Texture, Window,
};
use modor::{Component, ComponentSystems, Query, Single, SingleMut};
use std::fmt::Debug;
use std::ops::Range;
use wgpu::{IndexFormat, RenderPass};

pub(crate) type RenderTargetRegistry = ResourceRegistry<RenderTarget>;

#[must_use]
#[derive(Component, Debug)]
pub struct RenderTarget {
    pub background_color: Color,
    key: ResourceKey,
    window: Option<WindowTarget>,
    default_texture_key: ResourceKey,
}

#[systems]
impl RenderTarget {
    pub fn new(key: impl IntoResourceKey) -> Self {
        Self {
            background_color: Color::BLACK,
            key: key.into_key(),
            window: None,
            default_texture_key: TextureKey::Blank.into_key(),
        }
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    #[run_as(RenderTargetUpdate)]
    fn update_window_target(
        &mut self,
        window: Option<&mut Window>,
        module: Option<SingleMut<'_, GraphicsModule>>,
    ) {
        self.window = if let (Some(mut module), Some(window)) = (module, window) {
            self.window
                .take()
                .or_else(|| WindowTarget::new(window, &mut module))
                .and_then(|t| {
                    if window.handle_id() == t.handle_id() {
                        Some(t)
                    } else {
                        WindowTarget::new(window, &mut module)
                    }
                })
                .and_then(|t| t.updated(window, &module))
        } else {
            None
        };
    }

    #[run_after_previous_and(
        component(OpaqueInstanceRegistry),
        component(TransparentInstanceRegistry),
        component(Camera2DRegistry),
        component(Camera2D),
        component(MaterialRegistry),
        component(Material),
        component(ShaderRegistry),
        component(Shader),
        component(MeshRegistry),
        component(Mesh),
        component(TextureRegistry),
        component(Texture)
    )]
    #[allow(clippy::too_many_arguments)]
    fn render_window_target(
        &mut self,
        module: Single<'_, GraphicsModule>,
        opaque_instances: Single<'_, OpaqueInstanceRegistry>,
        transparent_instances: Single<'_, TransparentInstanceRegistry>,
        (mut camera_registry, cameras): (SingleMut<'_, Camera2DRegistry>, Query<'_, &Camera2D>),
        (mut material_registry, materials): (SingleMut<'_, MaterialRegistry>, Query<'_, &Material>),
        (mut shader_registry, shaders): (SingleMut<'_, ShaderRegistry>, Query<'_, &Shader>),
        (mut mesh_registry, meshes): (SingleMut<'_, MeshRegistry>, Query<'_, &Mesh>),
        (mut texture_registry, textures): (SingleMut<'_, TextureRegistry>, Query<'_, &Texture>),
    ) {
        if let Some(target) = &mut self.window {
            let mut pass = target.begin_render_pass(self.background_color, &module);
            for (group_key, instance_buffer) in opaque_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    group_key,
                    instance_buffer,
                    None,
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &self.default_texture_key,
                );
            }
            for (group_key, instance_buffer, instance_range) in transparent_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    group_key,
                    instance_buffer,
                    Some(instance_range),
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &self.default_texture_key,
                );
            }
            drop(pass);
            target.end_render_pass(&module);
        }
    }

    pub(crate) fn window_surface_size(&self) -> Option<NonZeroSize> {
        self.window.as_ref().map(|w| w.core().size())
    }

    #[allow(clippy::cast_possible_truncation, clippy::too_many_arguments)]
    fn draw<'a>(
        pass: &mut RenderPass<'a>,
        target_key: &ResourceKey,
        group_key: &GroupKey,
        instance_buffer: &'a DynamicBuffer<Instance>,
        instance_range: Option<Range<usize>>,
        (camera_registry, cameras): (&mut Camera2DRegistry, &'a Query<'_, &Camera2D>),
        (material_registry, materials): (&mut MaterialRegistry, &'a Query<'_, &Material>),
        (shader_registry, shaders): (&mut ShaderRegistry, &'a Query<'_, &Shader>),
        (mesh_registry, meshes): (&mut MeshRegistry, &'a Query<'_, &Mesh>),
        (texture_registry, textures): (&mut TextureRegistry, &'a Query<'_, &Texture>),
        default_texture_key: &ResourceKey,
    ) -> Option<()> {
        let camera = camera_registry.get(&group_key.camera_key, cameras)?;
        if !camera.target_keys.contains(target_key) {
            return None;
        }
        let material = material_registry.get(&group_key.material_key, materials)?;
        let shader = shader_registry.get(&material.shader_key, shaders)?;
        let mesh = mesh_registry.get(&group_key.mesh_key, meshes)?;
        let texture_key = material.texture_key.as_ref().unwrap_or(default_texture_key);
        let texture = texture_registry.get(texture_key, textures)?;
        let camera_uniform = camera.uniform(target_key);
        let material_uniform = material.uniform();
        let vertex_buffer = mesh.vertex_buffer();
        let index_buffer = mesh.index_buffer();
        pass.set_pipeline(shader.pipeline());
        pass.set_bind_group(Shader::CAMERA_GROUP, camera_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::MATERIAL_GROUP, material_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::TEXTURE_GROUP, texture.bind_group(), &[]);
        pass.set_vertex_buffer(0, vertex_buffer.buffer());
        pass.set_vertex_buffer(1, instance_buffer.buffer());
        pass.set_index_buffer(index_buffer.buffer(), IndexFormat::Uint16);
        pass.draw_indexed(
            0..(index_buffer.len() as u32),
            0,
            (instance_range.clone().map_or(0, |r| r.start) as u32)
                ..(instance_range.map_or_else(|| instance_buffer.len(), |r| r.end) as u32),
        );
        Some(())
    }
}

impl Resource for RenderTarget {
    fn key(&self) -> &ResourceKey {
        &self.key
    }

    fn state(&self) -> ResourceState<'_> {
        if self.window.is_some() {
            ResourceState::Loaded
        } else {
            ResourceState::NotLoaded
        }
    }
}

#[derive(Action)]
pub(crate) struct RenderTargetUpdate(<Window as ComponentSystems>::Action);

mod core;
mod window;
