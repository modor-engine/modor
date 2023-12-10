use crate::components::camera::Camera2DRegistry;
use crate::components::instance_group::InstanceGroup2DRegistry;
use crate::components::material::MaterialRegistry;
use crate::components::mesh::{Mesh, MeshRegistry};
use crate::components::render_target::texture::TextureTarget;
use crate::components::render_target::window::WindowTarget;
use crate::components::shader::{Shader, ShaderRegistry};
use crate::components::texture::{TextureRegistry, INVISIBLE_TEXTURE, WHITE_TEXTURE};
use crate::data::size::NonZeroSize;
use crate::{
    errors, AntiAliasing, Camera2D, Color, FrameRate, InstanceGroup2D, InstanceRendering2D,
    Material, Renderer, Texture, Window,
};
use itertools::Itertools;
use modor::{Component, ComponentSystems, Custom, Query, Single, SingleRef};
use modor_resources::{
    ResKey, Resource, ResourceAccessor, ResourceLoadingError, ResourceRegistry, ResourceState,
};
use std::fmt::Debug;
use wgpu::{IndexFormat, RenderPass, TextureFormat};

pub(crate) type RenderTargetRegistry = ResourceRegistry<RenderTarget>;

/// The target for a rendering.
///
/// If a [`Window`] component is in the same entity, then the rendering is performed in this window.
///
/// If a [`Texture`] component is in the same entity, then the rendering is performed in this
/// texture. This texture can then be displayed in another render target.
/// If the texture is used in its own render target, then the attached instances are not displayed.
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
/// - [`texture_target`](crate::texture_target())
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
    is_rendering_error_logged: bool,
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
            is_rendering_error_logged: false,
            window_renderer_version: None,
            texture_renderer_version: None,
        }
    }

    #[run_as(action(WindowTargetUpdate))]
    fn update_window_target(
        &mut self,
        window: Option<&mut Window>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        frame_rate: Option<SingleRef<'_, '_, FrameRate>>,
        anti_aliasing: Option<SingleRef<'_, '_, AntiAliasing>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.window_renderer_version);
        if state.is_removed() || window.is_none() {
            self.window = None;
        }
        let anti_aliasing = anti_aliasing.as_ref().map(SingleRef::get);
        if let (Some(context), Some(window)) = (state.context(), window) {
            let frame_rate = frame_rate
                .as_ref()
                .map(Single::get)
                .copied()
                .unwrap_or_default();
            self.window = self
                .window
                .take()
                .or_else(|| WindowTarget::new(window, anti_aliasing, context))
                .map(|t| t.updated(window, context, frame_rate, anti_aliasing));
        }
        self.window_state = if self.window.is_some() {
            TargetState::Loaded
        } else {
            TargetState::NotLoaded
        };
    }

    #[run_as(action(TextureTargetUpdate))]
    fn update_texture_target(
        &mut self,
        texture: Option<&Texture>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
        anti_aliasing: Option<SingleRef<'_, '_, AntiAliasing>>,
    ) {
        let state = Renderer::option_state(&renderer, &mut self.texture_renderer_version);
        if state.is_removed() || texture.is_none() {
            self.texture = None;
        }
        let anti_aliasing = anti_aliasing.as_ref().map(SingleRef::get);
        if let (Some(context), Some(texture)) = (state.context(), texture) {
            self.texture = (texture.state() == ResourceState::Loaded).then(|| {
                self.texture
                    .take()
                    .unwrap_or_else(|| TextureTarget::new(texture, anti_aliasing, context))
                    .updated(texture, anti_aliasing, context)
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
        action(WindowTargetUpdate),
        action(TextureTargetUpdate),
        component(Renderer),
        component(InstanceRendering2D),
        component(InstanceGroup2DRegistry),
        component(InstanceGroup2D),
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
    fn render(
        &mut self,
        texture: Option<&Texture>,
        renderer: SingleRef<'_, '_, Renderer>,
        instance_renderings: Query<'_, &InstanceRendering2D>,
        resources: Custom<RenderingResources<'_>>,
    ) {
        let Some(context) = renderer.get().state(&mut None).context() else {
            return; // no-coverage (cannot be easily tested)
        };
        self.window = if let Some(mut target) = self.window.take() {
            let target_texture_format = context
                .surface_texture_format
                .expect("internal error: cannot determine window format");
            let mut pass = target.begin_render_pass(self.background_color, context);
            for rendering in Self::sorted_opaque_renderings(&instance_renderings) {
                self.render_instances(
                    &mut pass,
                    RenderingMode::Opaque,
                    None,
                    target_texture_format,
                    TargetType::Window,
                    rendering,
                    &resources,
                );
            }
            for (rendering, instance_group, i) in
                Self::sorted_transparent_instances(&instance_renderings, &resources)
            {
                self.render_instances(
                    &mut pass,
                    RenderingMode::Transparent(instance_group, i),
                    None,
                    target_texture_format,
                    TargetType::Window,
                    rendering,
                    &resources,
                );
            }
            let result = errors::validate_wgpu(context, || drop(pass));
            target.end_render_pass(context, result.is_ok());
            trace!(
                "rendering done in window target `{}` (error: {})", // no-coverage
                self.key.label(),                                   // no-coverage
                result.is_err()                                     // no-coverage
            );
            self.log_rendering_error(result);
            Some(target)
        } else {
            None
        };
        self.texture = if let (Some(mut target), Some(texture)) = (self.texture.take(), texture) {
            let mut pass = target.begin_render_pass(texture, self.background_color, context);
            for instance_rendering in Self::sorted_opaque_renderings(&instance_renderings) {
                self.render_instances(
                    &mut pass,
                    RenderingMode::Opaque,
                    Some(texture.key()),
                    Shader::TEXTURE_FORMAT,
                    TargetType::Texture,
                    instance_rendering,
                    &resources,
                );
            }
            for (rendering, instance_group, i) in
                Self::sorted_transparent_instances(&instance_renderings, &resources)
            {
                self.render_instances(
                    &mut pass,
                    RenderingMode::Transparent(instance_group, i),
                    Some(texture.key()),
                    Shader::TEXTURE_FORMAT,
                    TargetType::Texture,
                    rendering,
                    &resources,
                );
            }
            let result = errors::validate_wgpu(context, || drop(pass));
            target.end_render_pass(context, result.is_ok());
            trace!(
                "rendering done in texture target `{}` (error: {})", // no-coverage
                self.key.label(),                                    // no-coverage
                result.is_err()                                      // no-coverage
            );
            self.log_rendering_error(result);
            Some(target)
        } else {
            None
        };
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
    fn render_instances<'a>(
        &mut self,
        pass: &mut RenderPass<'a>,
        mode: RenderingMode<'a>,
        target_texture_key: Option<ResKey<Texture>>,
        target_texture_format: TextureFormat,
        target_type: TargetType,
        rendering: &InstanceRendering2D,
        resources: &'a Custom<RenderingResources<'_>>,
    ) -> Option<()> {
        let camera = resources.cameras.get(rendering.camera_key)?;
        if !camera.target_keys.contains(&self.key) {
            return None;
        }
        let material = resources.materials.get(rendering.material_key)?;
        let texture_key = material.texture_key.unwrap_or(WHITE_TEXTURE);
        let texture = self.texture(
            rendering.material_key,
            texture_key,
            target_texture_key,
            resources,
        )?;
        let texture_bind_ground = &texture.inner().bind_group;
        let front_texture_key = material.front_texture_key.unwrap_or(INVISIBLE_TEXTURE);
        let front_texture = self.texture(
            rendering.material_key,
            front_texture_key,
            target_texture_key,
            resources,
        )?;
        let front_texture_bind_ground = &front_texture.inner().bind_group;
        let shader = resources.shaders.get(material.shader_key)?;
        let mesh = resources.meshes.get(rendering.mesh_key)?;
        let camera_uniform = camera.uniform(self.key, target_type);
        let material_uniform = material.uniform();
        let vertex_buffer = mesh.vertex_buffer();
        let index_buffer = mesh.index_buffer();
        pass.set_pipeline(shader.pipeline(target_texture_format));
        pass.set_bind_group(Shader::CAMERA_GROUP, camera_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::MATERIAL_GROUP, material_uniform.bind_group(), &[]);
        pass.set_bind_group(Shader::TEXTURE_GROUP, texture_bind_ground, &[]);
        pass.set_bind_group(Shader::FRONT_TEXTURE_GROUP, front_texture_bind_ground, &[]);
        pass.set_index_buffer(index_buffer.buffer(), IndexFormat::Uint16);
        pass.set_vertex_buffer(0, vertex_buffer.buffer());
        match mode {
            RenderingMode::Opaque => {
                let instance_group = resources.instance_groups.get(rendering.group_key)?;
                pass.set_vertex_buffer(1, instance_group.buffer().buffer());
                pass.draw_indexed(
                    0..(index_buffer.len() as u32),
                    0,
                    0..(instance_group.buffer().len() as u32),
                );
            }
            RenderingMode::Transparent(instance_group, i) => {
                pass.set_vertex_buffer(1, instance_group.buffer().buffer());
                pass.draw_indexed(
                    0..(index_buffer.len() as u32),
                    0,
                    i as u32..((i + 1) as u32),
                );
            }
        }
        Some(())
    }

    fn texture<'a>(
        &mut self,
        material_key: ResKey<Material>,
        texture_key: ResKey<Texture>,
        target_texture_key: Option<ResKey<Texture>>,
        resources: &'a Custom<RenderingResources<'_>>,
    ) -> Option<&'a Texture> {
        if target_texture_key == Some(texture_key) {
            if !self.is_texture_conflict_logged {
                error!(
                    "texture `{}` cannot be used to render instance in render target `{}` with \
                    material `{}` because the texture is the target",
                    texture_key.label(),
                    self.key.label(),
                    material_key.label(),
                );
                self.is_texture_conflict_logged = true;
            }
            return None;
        }
        resources.textures.get(texture_key)
    }

    fn sorted_opaque_renderings<'a>(
        instance_renderings: &'a Query<'_, &InstanceRendering2D>,
    ) -> impl Iterator<Item = &'a InstanceRendering2D> {
        instance_renderings
            .iter()
            .filter(|rendering| !rendering.is_transparent)
            .sorted_unstable()
    }

    fn sorted_transparent_instances<'a>(
        instance_renderings: &'a Query<'_, &InstanceRendering2D>,
        resources: &'a Custom<RenderingResources<'_>>,
    ) -> impl Iterator<Item = (&'a InstanceRendering2D, &'a InstanceGroup2D, usize)> {
        instance_renderings
            .iter()
            .filter(|rendering| rendering.is_transparent)
            .filter_map(|rendering| {
                resources
                    .instance_groups
                    .get(rendering.group_key)
                    .map(|group| (rendering, group))
            })
            .flat_map(|(rendering, group)| {
                (0..group.buffer().len()).map(move |i| (rendering, group, i))
            })
            .sorted_unstable_by(|(rendering1, group1, i1), (rendering2, group2, i2)| {
                let z1 = group1.buffer()[*i1].z();
                let z2 = &group2.buffer()[*i2].z();
                z1.total_cmp(z2).then(rendering1.cmp(rendering2))
            })
    }

    fn log_rendering_error(&mut self, result: Result<(), wgpu::Error>) {
        if !self.is_rendering_error_logged {
            if let Err(error) = result {
                error!(
                    "error while rendering target `{}`: {error}",
                    self.key.label()
                );
                self.is_rendering_error_logged = true;
            }
        }
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
    <AntiAliasing as ComponentSystems>::Action,
);

#[derive(Action)]
pub(crate) struct TextureTargetUpdate(
    <Texture as ComponentSystems>::Action,
    <Renderer as ComponentSystems>::Action,
    <AntiAliasing as ComponentSystems>::Action,
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

#[derive(SystemParam)]
struct RenderingResources<'a> {
    instance_groups: Custom<ResourceAccessor<'a, InstanceGroup2D>>,
    cameras: Custom<ResourceAccessor<'a, Camera2D>>,
    materials: Custom<ResourceAccessor<'a, Material>>,
    shaders: Custom<ResourceAccessor<'a, Shader>>,
    meshes: Custom<ResourceAccessor<'a, Mesh>>,
    textures: Custom<ResourceAccessor<'a, Texture>>,
}

#[derive(Clone, Copy, Debug)]
enum RenderingMode<'a> {
    Opaque,
    Transparent(&'a InstanceGroup2D, usize),
}

mod core;
mod texture;
mod window;
