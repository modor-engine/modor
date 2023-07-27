use crate::components::camera::Camera2DRegistry;
use crate::components::instances::opaque::OpaqueInstanceRegistry;
use crate::components::instances::transparent::TransparentInstanceRegistry;
use crate::components::instances::{GroupKey, Instance};
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::texture::TextureTarget;
use crate::components::render_target::window::WindowTarget;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::data::size::NonZeroSize;
use crate::gpu_data::buffer::DynamicBuffer;
use crate::{Camera2D, Color, FrameRate, Material, Renderer, Texture, Window};
use modor::{Component, ComponentSystems, Query, Single, SingleMut, SingleRef};
use modor_resources::{ResKey, Resource, ResourceLoadingError, ResourceRegistry, ResourceState};
use std::fmt::Debug;
use std::ops::Range;
use wgpu::{IndexFormat, RenderPass, TextureFormat};

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
/// # Entity functions creating this component
///
/// - [`window_target`](crate::window_target())
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_physics::*;
/// # use modor_math::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// const CAMERA: ResKey<Camera2D> = ResKey::new("main");
/// const TARGET_TEXTURE: ResKey<Texture> = ResKey::new("target");
/// const WINDOW_TARGET: ResKey<RenderTarget> = ResKey::new("window");
/// const TEXTURE_TARGET: ResKey<RenderTarget> = ResKey::new("texture");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_entity(window_target())
///         .child_entity(texture_target())
///         .child_component(Camera2D::new(CAMERA, WINDOW_TARGET))
///         .with(|c| c.target_keys.push(TEXTURE_TARGET))
/// }
///
/// fn window_target() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Window::default())
///         .component(RenderTarget::new(WINDOW_TARGET))
/// }
///
/// fn texture_target() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Texture::from_size(TARGET_TEXTURE, Size::new(800, 600)))
///         .component(RenderTarget::new(TEXTURE_TARGET))
///         .with(|t| t.background_color = Color::GREEN)
/// }
/// ```
#[must_use]
#[derive(Component, Debug)]
pub struct RenderTarget {
    /// Background color of the rendering.
    ///
    /// Default is [`Color::BLACK`].
    pub background_color: Color,
    key: ResKey<Self>,
    window: Option<WindowTarget>,
    texture: Option<TextureTarget>,
    window_state: TargetState,
    texture_state: TargetState,
    is_texture_conflict_logged: bool,
    window_renderer_version: Option<u8>,
    texture_renderer_version: Option<u8>,
}

#[systems]
impl RenderTarget {
    /// Creates a new target with a unique `key`.
    pub fn new(key: ResKey<Self>) -> Self {
        Self {
            background_color: Color::BLACK,
            key,
            window: None,
            texture: None,
            window_state: TargetState::NotLoaded,
            texture_state: TargetState::NotLoaded,
            is_texture_conflict_logged: false,
            window_renderer_version: None,
            texture_renderer_version: None,
        }
    }

    #[run_as(WindowTargetUpdate)]
    fn update_window_target(
        &mut self,
        window: Option<&mut Window>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        frame_rate: Option<SingleRef<'_, '_, FrameRate>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.window_renderer_version);
        if state.is_removed() || window.is_none() {
            self.window = None;
        }
        if let (Some(context), Some(window)) = (state.context(), window) {
            let frame_rate = frame_rate
                .as_ref()
                .map(Single::get)
                .copied()
                .unwrap_or_default();
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
        renderer: Option<SingleRef<'_, '_, Renderer>>,
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
                    ResourceState::Loading => TargetState::Loading,
                    ResourceState::Error(e) => TargetState::Error(e.clone()),
                    ResourceState::NotLoaded => {
                        unreachable!("internal error: renderer existing but texture not loaded")
                    }
                    ResourceState::Loaded => {
                        unreachable!("internal error: target texture loaded but target not loaded")
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
        renderer: SingleRef<'_, '_, Renderer>,
        opaque_instances: SingleRef<'_, '_, OpaqueInstanceRegistry>,
        transparent_instances: SingleRef<'_, '_, TransparentInstanceRegistry>,
        (mut camera_registry, cameras): (SingleMut<'_, '_, Camera2DRegistry>, Query<'_, &Camera2D>),
        (mut material_registry, materials): (
            SingleMut<'_, '_, MaterialRegistry>,
            Query<'_, &Material>,
        ),
        (mut shader_registry, shaders): (SingleMut<'_, '_, ShaderRegistry>, Query<'_, &Shader>),
        (mut mesh_registry, meshes): (SingleMut<'_, '_, MeshRegistry>, Query<'_, &Mesh>),
        (mut texture_registry, textures): (SingleMut<'_, '_, TextureRegistry>, Query<'_, &Texture>),
    ) {
        let Some(context) = renderer.get().state(&mut None).context() else { return; };
        if let Some(target) = &mut self.window {
            let target_texture_format = context
                .surface_texture_format
                .expect("internal error: cannot determine window format");
            let mut pass = target.begin_render_pass(self.background_color, context);
            for (group_key, instance_buffer) in opaque_instances.get().iter() {
                Self::draw(
                    &mut pass,
                    self.key,
                    TargetType::Window,
                    target_texture_format,
                    group_key,
                    None,
                    instance_buffer,
                    None,
                    (camera_registry.get_mut(), &cameras),
                    (material_registry.get_mut(), &materials),
                    (shader_registry.get_mut(), &shaders),
                    (mesh_registry.get_mut(), &meshes),
                    (texture_registry.get_mut(), &textures),
                    &mut self.is_texture_conflict_logged,
                );
            }
            for (group_key, instance_buffer, instance_range) in transparent_instances.get().iter() {
                Self::draw(
                    &mut pass,
                    self.key,
                    TargetType::Window,
                    target_texture_format,
                    group_key,
                    None,
                    instance_buffer,
                    Some(instance_range),
                    (camera_registry.get_mut(), &cameras),
                    (material_registry.get_mut(), &materials),
                    (shader_registry.get_mut(), &shaders),
                    (mesh_registry.get_mut(), &meshes),
                    (texture_registry.get_mut(), &textures),
                    &mut self.is_texture_conflict_logged,
                );
            }
            drop(pass);
            target.end_render_pass(context);
            trace!("rendering done in window target `{:?}`", self.key);
        }
        if let (Some(target), Some(texture)) = (&mut self.texture, texture) {
            let mut pass = target.begin_render_pass(texture, self.background_color, context);
            for (group_key, instance_buffer) in opaque_instances.get().iter() {
                Self::draw(
                    &mut pass,
                    self.key,
                    TargetType::Texture,
                    Shader::TEXTURE_FORMAT,
                    group_key,
                    Some(texture.key()),
                    instance_buffer,
                    None,
                    (camera_registry.get_mut(), &cameras),
                    (material_registry.get_mut(), &materials),
                    (shader_registry.get_mut(), &shaders),
                    (mesh_registry.get_mut(), &meshes),
                    (texture_registry.get_mut(), &textures),
                    &mut self.is_texture_conflict_logged,
                );
            }
            for (group_key, instance_buffer, instance_range) in transparent_instances.get().iter() {
                Self::draw(
                    &mut pass,
                    self.key,
                    TargetType::Texture,
                    Shader::TEXTURE_FORMAT,
                    group_key,
                    Some(texture.key()),
                    instance_buffer,
                    Some(instance_range),
                    (camera_registry.get_mut(), &cameras),
                    (material_registry.get_mut(), &materials),
                    (shader_registry.get_mut(), &shaders),
                    (mesh_registry.get_mut(), &meshes),
                    (texture_registry.get_mut(), &textures),
                    &mut self.is_texture_conflict_logged,
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
        target_key: ResKey<Self>,
        target_type: TargetType,
        target_texture_format: TextureFormat,
        group_key: GroupKey,
        target_texture_key: Option<ResKey<Texture>>,
        instance_buffer: &'a DynamicBuffer<Instance>,
        instance_range: Option<Range<usize>>,
        (camera_registry, cameras): (&mut Camera2DRegistry, &'a Query<'_, &Camera2D>),
        (material_registry, materials): (&mut MaterialRegistry, &'a Query<'_, &Material>),
        (shader_registry, shaders): (&mut ShaderRegistry, &'a Query<'_, &Shader>),
        (mesh_registry, meshes): (&mut MeshRegistry, &'a Query<'_, &Mesh>),
        (texture_registry, textures): (&mut TextureRegistry, &'a Query<'_, &Texture>),
        is_texture_conflict_logged: &mut bool,
    ) -> Option<()> {
        let camera = camera_registry.get(group_key.camera_key, cameras)?;
        if !camera.target_keys.contains(&target_key) {
            return None;
        }
        let material = material_registry.get(group_key.material_key, materials)?;
        let texture_key = material.texture_key.unwrap_or(WHITE_TEXTURE);
        let texture = Self::texture(
            target_key,
            group_key,
            texture_key,
            target_texture_key,
            (texture_registry, textures),
            is_texture_conflict_logged,
        )?;
        let texture_bind_ground = &texture.inner().bind_group;
        let front_texture_key = material.front_texture_key.unwrap_or(INVISIBLE_TEXTURE);
        let front_texture = Self::texture(
            target_key,
            group_key,
            front_texture_key,
            target_texture_key,
            (texture_registry, textures),
            is_texture_conflict_logged,
        )?;
        let front_texture_bind_ground = &front_texture.inner().bind_group;
        let shader = shader_registry.get(material.shader_key, shaders)?;
        let mesh = mesh_registry.get(group_key.mesh_key, meshes)?;
        let camera_uniform = camera.uniform(target_key, target_type);
        let material_uniform = material.uniform();
        let vertex_buffer = mesh.vertex_buffer();
        let index_buffer = mesh.index_buffer();
        pass.set_pipeline(shader.pipeline(target_texture_format));
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
        target_key: ResKey<Self>,
        group_key: GroupKey,
        texture_key: ResKey<Texture>,
        target_texture_key: Option<ResKey<Texture>>,
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
    fn key(&self) -> ResKey<Self> {
        self.key
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
