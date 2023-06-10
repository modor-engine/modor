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
use crate::{Camera2D, Color, FrameRate, Material, Renderer, Texture, Window};
use modor::{Component, ComponentSystems, Query, Single, SingleMut};
use modor_resources::{
    IntoResourceKey, Resource, ResourceKey, ResourceLoadingError, ResourceRegistry, ResourceState,
};
use std::fmt::Debug;
use std::ops::Range;
use wgpu::{IndexFormat, RenderPass};

pub(crate) type RenderTargetRegistry = ResourceRegistry<RenderTarget>;

/// The target for a rendering.
///
/// If a [`Window`] component is in the same entity, then the rendering is performed in this window.
///
/// If a [`Texture`] component is in the same entity, then the rendering is performed in this
/// texture. This texture can then be displayed in another render target.
/// If the texture is used in its own render target, then the attached models are not displayed.
///
/// # Requirements
///
/// The rendering is performed only if:
/// - graphics [`module`](crate::module()) is initialized
/// - either [`Window`] or [`Texture`] component is in the same entity
///
/// # Related components
///
/// - [`Window`]
/// - [`Texture`]
/// - [`Camera2D`]
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// # use modor_graphics_new2::*;
/// #
/// fn root() -> impl BuiltEntity {
///     let camera = Camera2D::new(CameraKey)
///         .with_target_key(TargetKey::Window)
///         .with_target_key(TargetKey::Texture);
///     EntityBuilder::new()
///         .with_child(window_target())
///         .with_child(texture_target())
///         .with_child(camera)
/// }
///
/// fn window_target() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Window::default())
///         .with(RenderTarget::new(TargetKey::Texture))
/// }
///
/// fn texture_target() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Texture::from_size(TextureKey, Size::new(800, 600)))
///         .with(RenderTarget::new(TargetKey::Texture).with_background_color(Color::GREEN))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum TargetKey {
///     Window,
///     Texture,
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct TextureKey;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
/// ```
#[must_use]
#[derive(Component, Debug)]
pub struct RenderTarget {
    /// Background color of the rendering.
    ///
    /// Default is [`Color::BLACK`].
    pub background_color: Color,
    key: ResourceKey,
    window: Option<WindowTarget>,
    texture: Option<TextureTarget>,
    window_state: TargetState,
    texture_state: TargetState,
    is_texture_conflict_logged: bool,
    default_texture_key: ResourceKey,
    default_front_texture_key: ResourceKey,
    window_renderer_version: Option<u8>,
    texture_renderer_version: Option<u8>,
}

#[systems]
impl RenderTarget {
    /// Creates a new target with a unique `key`.
    pub fn new(key: impl IntoResourceKey) -> Self {
        Self {
            background_color: Color::BLACK,
            key: key.into_key(),
            window: None,
            texture: None,
            window_state: TargetState::NotLoaded,
            texture_state: TargetState::NotLoaded,
            is_texture_conflict_logged: false,
            default_texture_key: TextureKey::White.into_key(),
            default_front_texture_key: TextureKey::Invisible.into_key(),
            window_renderer_version: None,
            texture_renderer_version: None,
        }
    }

    /// Returns the target with a given [`background_color`](#structfield.background_color).
    pub fn with_background_color(mut self, background_color: Color) -> Self {
        self.background_color = background_color;
        self
    }

    #[run_as(WindowTargetUpdate)]
    fn update_window_target(
        &mut self,
        window: Option<&mut Window>,
        renderer: Option<Single<'_, Renderer>>,
        frame_rate: Option<Single<'_, FrameRate>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.window_renderer_version);
        if state.is_removed() || window.is_none() {
            self.window = None;
        }
        if let (Some(context), Some(window)) = (state.context(), window) {
            let frame_rate = frame_rate.as_deref().copied().unwrap_or_default();
            self.window = self
                .window
                .take()
                .or_else(|| WindowTarget::new(window, context))
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
        let state = Renderer::option_state(&renderer, &mut self.texture_renderer_version);
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
    fn render(
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
                    &self.default_front_texture_key,
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
                    &self.default_front_texture_key,
                );
            }
            drop(pass);
            target.end_render_pass(context);
            trace!("rendering done in window target `{:?}`", self.key);
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
                    &self.default_front_texture_key,
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
                    &self.default_front_texture_key,
                );
            }
            drop(pass);
            target.end_render_pass(context);
            trace!("rendering done in texture target `{:?}`", self.key);
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
        default_front_texture_key: &ResourceKey,
    ) -> Option<()> {
        let camera = camera_registry.get(&group_key.camera_key, cameras)?;
        if !camera.target_keys.contains(target_key) {
            return None;
        }
        let material = material_registry.get(&group_key.material_key, materials)?;
        let texture_key = material.texture_key.as_ref().unwrap_or(default_texture_key);
        let texture = Self::texture(
            target_key,
            group_key,
            texture_key,
            target_texture_key,
            (texture_registry, textures),
            is_texture_conflict_logged,
        )?;
        let texture_bind_ground = &texture.inner().bind_group;
        let front_texture_key = material
            .front_texture_key
            .as_ref()
            .unwrap_or(default_front_texture_key);
        let front_texture = Self::texture(
            target_key,
            group_key,
            front_texture_key,
            target_texture_key,
            (texture_registry, textures),
            is_texture_conflict_logged,
        )?;
        let front_texture_bind_ground = &front_texture.inner().bind_group;
        let shader = shader_registry.get(&material.shader_key, shaders)?;
        let mesh = mesh_registry.get(&group_key.mesh_key, meshes)?;
        let camera_uniform = camera.uniform(target_key, target_type);
        let material_uniform = material.uniform();
        let vertex_buffer = mesh.vertex_buffer();
        let index_buffer = mesh.index_buffer();
        pass.set_pipeline(shader.pipeline());
        pass.set_bind_group(Shader::CAMERA_GROUP, camera_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::MATERIAL_GROUP, material_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::TEXTURE_GROUP, texture_bind_ground, &[]);
        pass.set_bind_group(Shader::FRONT_TEXTURE_GROUP, front_texture_bind_ground, &[]);
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

    fn texture<'a>(
        target_key: &ResourceKey,
        group_key: &GroupKey,
        texture_key: &ResourceKey,
        target_texture_key: Option<&ResourceKey>,
        (texture_registry, textures): (&mut TextureRegistry, &'a Query<'_, &Texture>),
        is_texture_conflict_logged: &mut bool,
    ) -> Option<&'a Texture> {
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
        texture_registry.get(texture_key, textures)
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
