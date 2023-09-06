use crate::components::render_target::{
    RenderTargetRegistry, TargetType, TextureTargetUpdate, WindowTargetUpdate,
};
use crate::data::size::NonZeroSize;
use crate::gpu_data::uniform::Uniform;
use crate::{GpuContext, RenderTarget, Renderer, Size};
use fxhash::FxHashMap;
use modor::{Custom, SingleRef};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource, ResourceAccessor, ResourceRegistry, ResourceState};

pub(crate) type Camera2DRegistry = ResourceRegistry<Camera2D>;

/// A camera used for 2D rendering.
///
/// By default, camera displays a world zone centered in position [`Vec2::ZERO`] and with size
/// [`Vec2::ONE`]. If the render target width is different than its height, more parts of the world
/// might be rendered, but the focused zone is always entirely displayed.
///
/// The displayed zone can be configured with a [`Transform2D`] created in the same entity.
///
/// # Requirements
///
/// The component is effective only if:
/// - graphics [`module`](crate::module()) is initialized
///
/// # Related components
///
/// - [`Transform2D`]
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
/// const DEFAULT_CAMERA: ResKey<Camera2D> = ResKey::new("default");
/// const DYNAMIC_CAMERA: ResKey<Camera2D> = ResKey::new("dynamic");
/// const TARGET: ResKey<RenderTarget> = ResKey::new("main");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_entity(render_target())
///         .child_component(Camera2D::new(DEFAULT_CAMERA, TARGET))
///         .child_entity(dynamic_camera())
///         .child_entity(object())
/// }
///
/// fn render_target() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Window::default())
///         .component(RenderTarget::new(TARGET))
/// }
///
/// fn dynamic_camera() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Camera2D::new(DYNAMIC_CAMERA, TARGET))
///         .component(Transform2D::new())
///         .with(|t| *t.size = Vec2::ONE * 0.5) // zoom x2
///         .component(Dynamics2D::new())
///         .with(|d| *d.velocity = Vec2::new(0.1, 0.2))
/// }
///
/// fn object() -> impl BuiltEntity {
///     model_2d(DYNAMIC_CAMERA, Model2DMaterial::Rectangle)
///         .updated(|m: &mut Model| m.camera_keys.push(DEFAULT_CAMERA)) // add other camera
///         .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.3, 0.1))
/// }
/// ```
#[must_use]
#[derive(Component, Debug)]
pub struct Camera2D {
    /// Keys of the [`RenderTarget`]s where the camera should be used.
    ///
    /// If the camera is used for a target, all associated [`Model`](crate::Model)s will be rendered
    /// in this target.
    pub target_keys: Vec<ResKey<RenderTarget>>,
    key: ResKey<Self>,
    transform: Transform2D,
    target_uniforms: FxHashMap<TargetPartKey, Uniform<CameraData>>,
    renderer_version: Option<u8>,
}

#[systems]
impl Camera2D {
    const CAMERA_BINDING: u32 = 0;

    /// Creates a new camera with a unique `key` and linked to a
    /// [`RenderTarget`](RenderTarget).
    pub fn new(key: ResKey<Self>, target_key: ResKey<RenderTarget>) -> Self {
        Self {
            target_keys: vec![target_key],
            key,
            transform: Transform2D::default(),
            target_uniforms: FxHashMap::default(),
            renderer_version: None,
        }
    }

    /// Creates a new camera with a unique `key` and not linked to
    /// a [`RenderTarget`](RenderTarget).
    pub fn hidden(key: ResKey<Self>) -> Self {
        Self {
            target_keys: vec![],
            key,
            transform: Transform2D::default(),
            target_uniforms: FxHashMap::default(),
            renderer_version: None,
        }
    }

    #[run_after(
        action(WindowTargetUpdate),
        action(TextureTargetUpdate),
        component(Transform2D),
        component(RenderTargetRegistry),
        component(Renderer)
    )]
    fn update(
        &mut self,
        transform: Option<&Transform2D>,
        targets: Custom<ResourceAccessor<'_, RenderTarget>>,
        renderer: Option<SingleRef<'_, '_, Renderer>>,
    ) {
        self.transform = transform.cloned().unwrap_or_default();
        let state = Renderer::option_state(&renderer, &mut self.renderer_version);
        if state.is_removed() {
            self.target_uniforms.clear();
        }
        if let Some(context) = state.context() {
            for &target_key in &self.target_keys {
                let target = targets.get(target_key);
                for (surface_size, target_type) in target.iter().flat_map(|t| t.surface_sizes()) {
                    let target_part_key = TargetPartKey {
                        target_key,
                        type_: target_type,
                    };
                    let transform = self.gpu_matrix(surface_size).to_array();
                    self.target_uniforms
                        .entry(target_part_key)
                        .and_modify(|u| u.transform = transform)
                        .or_insert_with(|| Self::create_uniform(context, transform));
                    trace!(
                        "2D camera `{:?}` prepared for target `{:?}` of type `{:?}`",
                        self.key,
                        target_key,
                        target_type
                    );
                }
            }
            self.target_uniforms.retain(|_, u| u.is_changed());
            for uniform in self.target_uniforms.values_mut() {
                uniform.sync(context);
            }
        }
    }

    /// Converts a `surface_position` for a surface of size `surface_size` into world position.
    ///
    /// `surface_position` with a value of [`Vec2::ZERO`] corresponds to top-left corner of the
    /// surface, and a value of [`Window::size()`](crate::Window::size) corresponds to the
    /// bottom-right corner.
    pub fn world_position(&self, surface_size: Size, surface_position: Vec2) -> Vec2 {
        let target_size: Vec2 = surface_size.into();
        self.world_matrix(target_size)
            * Vec2::new(
                surface_position.x / target_size.x - 0.5,
                0.5 - surface_position.y / target_size.y,
            )
    }

    pub(crate) fn uniform(
        &self,
        target_key: ResKey<RenderTarget>,
        target_type: TargetType,
    ) -> &Uniform<CameraData> {
        self.target_uniforms
            .get(&TargetPartKey {
                target_key,
                type_: target_type,
            })
            .expect("internal error: camera uniform not initialized")
    }

    fn create_uniform(context: &GpuContext, transform: [[f32; 4]; 4]) -> Uniform<CameraData> {
        Uniform::new(
            CameraData { transform },
            Self::CAMERA_BINDING,
            &context.camera_bind_group_layout,
            "camera_2d",
            &context.device,
        )
    }

    fn gpu_matrix(&self, surface_size: NonZeroSize) -> Mat4 {
        let surface_size: Vec2 = surface_size.into();
        let x_scale = 1.0_f32.min(surface_size.y / surface_size.x);
        let y_scale = 1.0_f32.min(surface_size.x / surface_size.y);
        let position = Vec3::new(-self.transform.position.x, -self.transform.position.y, -1.);
        let scale = Vec3::new(
            2. * x_scale / self.transform.size.x,
            2. * y_scale / self.transform.size.y,
            -1.,
        );
        Mat4::from_position(position)
            * Quat::from_z(*self.transform.rotation).matrix()
            * Mat4::from_scale(scale)
    }

    fn world_matrix(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let scale = self
            .transform
            .size
            .with_scale(Vec2::new(1. / x_scale, 1. / y_scale));
        Mat4::from_scale(scale.with_z(1.))
            * Quat::from_z(-*self.transform.rotation).matrix()
            * Mat4::from_position(self.transform.position.with_z(0.))
    }
}

impl Resource for Camera2D {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, PartialEq, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct CameraData {
    pub(crate) transform: [[f32; 4]; 4],
}

#[derive(Debug, PartialEq, Eq, Hash)]
struct TargetPartKey {
    target_key: ResKey<RenderTarget>,
    type_: TargetType,
}
