use crate::components::camera::Camera2DRegistry;
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::instances::{GroupKey, Instance};
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::texture::TextureTarget;
use crate::components::render_target::window::WindowTarget;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureKey, TextureRegistry};
use crate::data::size::NonZeroSize;
use crate::gpu_data::buffer::DynamicBuffer;
use crate::{
    Camera2D, Color, FrameRate, IntoResourceKey, Material, Renderer, Resource, ResourceKey,
    ResourceLoadingError, ResourceRegistry, ResourceState, Texture, Window,
};
use log::error;
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
    texture: Option<TextureTarget>,
    window_state: TargetState,
    texture_state: TargetState,
    is_texture_conflict_logged: bool,
    default_texture_key: ResourceKey,
    renderer_version: Option<u8>,
}

#[systems]
impl RenderTarget {
    pub fn new(key: impl IntoResourceKey) -> Self {
        Self {
            background_color: Color::BLACK,
            key: key.into_key(),
            window: None,
            texture: None,
            window_state: TargetState::NotLoaded,
            texture_state: TargetState::NotLoaded,
            is_texture_conflict_logged: false,
            default_texture_key: TextureKey::Blank.into_key(),
            renderer_version: None,
        }
    }

    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    #[run_as(WindowTargetUpdate)]
    fn update_window_target(
        &mut self,
        window: Option<&mut Window>,
        renderer: Option<Single<'_, Renderer>>,
        frame_rate: Option<Single<'_, FrameRate>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() || window.is_none() {
            self.window = None;
        }
        if let (Some(context), Some(window)) = (state.context(), window) {
            let frame_rate = frame_rate.as_deref().copied().unwrap_or_default();
            self.window = self
                .window
                .take()
                .or_else(|| WindowTarget::new(window, context))
                .and_then(|t| {
                    if window.handle_id() == t.handle_id() {
                        Some(t)
                    } else {
                        WindowTarget::new(window, context)
                    }
                })
                .map(|t| t.updated(window, context, frame_rate));
        }
        self.window_state = if self.window.is_some() {
            TargetState::Loaded
        } else {
            TargetState::NotLoaded
        };
    }

    #[run_as(TextureTargetUpdate)]
    fn update_texture_target(
        &mut self,
        texture: Option<&Texture>,
        renderer: Option<Single<'_, Renderer>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() || texture.is_none() {
            self.texture = None;
        }
        if let (Some(context), Some(texture)) = (state.context(), texture) {
            self.texture = (texture.state() == ResourceState::Loaded).then(|| {
                self.texture
                    .take()
                    .unwrap_or_else(|| TextureTarget::new(texture, context))
                    .updated(texture, context)
            });
        }
        self.texture_state = if self.texture.is_some() {
            TargetState::Loaded
        } else {
            match (&renderer, &texture) {
                (None, _) | (_, None) => TargetState::NotLoaded,
                (Some(_), Some(texture)) => match texture.state() {
                    ResourceState::NotLoaded => TargetState::NotLoaded,
                    ResourceState::Loading => TargetState::Loading,
                    ResourceState::Error(e) => TargetState::Error(e.clone()),
                    ResourceState::Loaded => {
                        unreachable!("internal error: target texture loaded but not target")
                    }
                },
            }
        };
    }

    #[run_after(
        WindowTargetUpdate,
        TextureTargetUpdate,
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
        texture: Option<&Texture>,
        renderer: Single<'_, Renderer>,
        opaque_instances: Single<'_, OpaqueInstanceRegistry>,
        transparent_instances: Single<'_, TransparentInstanceRegistry>,
        (mut camera_registry, cameras): (SingleMut<'_, Camera2DRegistry>, Query<'_, &Camera2D>),
        (mut material_registry, materials): (SingleMut<'_, MaterialRegistry>, Query<'_, &Material>),
        (mut shader_registry, shaders): (SingleMut<'_, ShaderRegistry>, Query<'_, &Shader>),
        (mut mesh_registry, meshes): (SingleMut<'_, MeshRegistry>, Query<'_, &Mesh>),
        (mut texture_registry, textures): (SingleMut<'_, TextureRegistry>, Query<'_, &Texture>),
    ) {
        let Some(context) = renderer.state(&mut None).context() else { return; };
        if let Some(target) = &mut self.window {
            let mut pass = target.begin_render_pass(self.background_color, context);
            for (group_key, instance_buffer) in opaque_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    TargetType::Window,
                    group_key,
                    None,
                    instance_buffer,
                    None,
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &mut self.is_texture_conflict_logged,
                    &self.default_texture_key,
                );
            }
            for (group_key, instance_buffer, instance_range) in transparent_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    TargetType::Window,
                    group_key,
                    None,
                    instance_buffer,
                    Some(instance_range),
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &mut self.is_texture_conflict_logged,
                    &self.default_texture_key,
                );
            }
            drop(pass);
            target.end_render_pass(context);
        }
        if let (Some(target), Some(texture)) = (&mut self.texture, texture) {
            let mut pass = target.begin_render_pass(texture, self.background_color, context);
            for (group_key, instance_buffer) in opaque_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    TargetType::Texture,
                    group_key,
                    Some(texture.key()),
                    instance_buffer,
                    None,
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &mut self.is_texture_conflict_logged,
                    &self.default_texture_key,
                );
            }
            for (group_key, instance_buffer, instance_range) in transparent_instances.iter() {
                Self::draw(
                    &mut pass,
                    &self.key,
                    TargetType::Texture,
                    group_key,
                    Some(texture.key()),
                    instance_buffer,
                    Some(instance_range),
                    (&mut camera_registry, &cameras),
                    (&mut material_registry, &materials),
                    (&mut shader_registry, &shaders),
                    (&mut mesh_registry, &meshes),
                    (&mut texture_registry, &textures),
                    &mut self.is_texture_conflict_logged,
                    &self.default_texture_key,
                );
            }
            drop(pass);
            target.end_render_pass(context);
        }
    }

    pub(crate) fn surface_sizes(&self) -> impl Iterator<Item = (NonZeroSize, TargetType)> {
        [
            self.window
                .as_ref()
                .map(|t| (t.core().size(), TargetType::Window)),
            self.texture
                .as_ref()
                .map(|t| (t.core().size(), TargetType::Texture)),
        ]
        .into_iter()
        .flatten()
    }

    #[allow(clippy::cast_possible_truncation, clippy::too_many_arguments)]
    fn draw<'a>(
        pass: &mut RenderPass<'a>,
        target_key: &ResourceKey,
        target_type: TargetType,
        group_key: &GroupKey,
        target_texture_key: Option<&ResourceKey>,
        instance_buffer: &'a DynamicBuffer<Instance>,
        instance_range: Option<Range<usize>>,
        (camera_registry, cameras): (&mut Camera2DRegistry, &'a Query<'_, &Camera2D>),
        (material_registry, materials): (&mut MaterialRegistry, &'a Query<'_, &Material>),
        (shader_registry, shaders): (&mut ShaderRegistry, &'a Query<'_, &Shader>),
        (mesh_registry, meshes): (&mut MeshRegistry, &'a Query<'_, &Mesh>),
        (texture_registry, textures): (&mut TextureRegistry, &'a Query<'_, &Texture>),
        is_texture_conflict_logged: &mut bool,
        default_texture_key: &ResourceKey,
    ) -> Option<()> {
        let camera = camera_registry.get(&group_key.camera_key, cameras)?;
        if !camera.target_keys.contains(target_key) {
            return None;
        }
        let material = material_registry.get(&group_key.material_key, materials)?;
        let texture_key = material.texture_key.as_ref().unwrap_or(default_texture_key);
        if target_texture_key == Some(texture_key) {
            if !*is_texture_conflict_logged {
                error!(
                    "texture `{:?}` used at same time as render target `{:?}` and material `{:?}`",
                    texture_key, target_key, group_key.material_key,
                );
                *is_texture_conflict_logged = true;
            }
            return None;
        }
        let texture = texture_registry.get(texture_key, textures)?;
        let shader = shader_registry.get(&material.shader_key, shaders)?;
        let mesh = mesh_registry.get(&group_key.mesh_key, meshes)?;
        let camera_uniform = camera.uniform(target_key, target_type);
        let material_uniform = material.uniform();
        let vertex_buffer = mesh.vertex_buffer();
        let index_buffer = mesh.index_buffer();
        pass.set_pipeline(shader.pipeline());
        pass.set_bind_group(Shader::CAMERA_GROUP, camera_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::MATERIAL_GROUP, material_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::TEXTURE_GROUP, &texture.inner().bind_group, &[]);
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
        match (&self.window_state, &self.texture_state) {
            (TargetState::Error(e), _) | (_, TargetState::Error(e)) => ResourceState::Error(e),
            (TargetState::Loading, _) | (_, TargetState::Loading) => ResourceState::Loading,
            (TargetState::Loaded, _) | (_, TargetState::Loaded) => ResourceState::Loaded,
            (TargetState::NotLoaded, TargetState::NotLoaded) => ResourceState::NotLoaded,
        }
    }
}

#[derive(Action)]
pub(crate) struct WindowTargetUpdate(
    <Window as ComponentSystems>::Action,
    <Renderer as ComponentSystems>::Action,
    <FrameRate as ComponentSystems>::Action,
);

#[derive(Action)]
pub(crate) struct TextureTargetUpdate(
    <Texture as ComponentSystems>::Action,
    <Renderer as ComponentSystems>::Action,
);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum TargetType {
    Window,
    Texture,
}

#[derive(Debug)]
enum TargetState {
    NotLoaded,
    Loading,
    Loaded,
    Error(ResourceLoadingError),
}

mod core;
mod texture;
mod window;
