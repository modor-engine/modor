use crate::buffer::{Buffer, BufferBindGroup};
use crate::gpu::{Gpu, GpuManager};
use crate::{Size, Target};
use fxhash::FxHashMap;
use modor::{App, Builder, FromApp, Glob, GlobRef, Global};
use modor_physics::modor_math::{Mat4, Quat, Vec2, Vec3};
use std::collections::hash_map::Entry;
use wgpu::{BindGroup, BufferUsages};

/// A camera used for 2D rendering.
///
/// By default, camera displays a world zone centered in position [`Vec2::ZERO`] and with size
/// [`Vec2::ONE`]. If the render target width is different from its height, more parts of the world
/// might be rendered, but the focused zone is always entirely displayed.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_graphics::modor_resources::*;
/// # use modor_physics::modor_math::*;
/// #
/// struct Object {
///     sprite: Sprite2D
/// }
///
/// impl Object {
///     fn new(app: &mut App) -> Self {
///         let camera = app.get_mut::<MovingCamera>().camera.glob().to_ref();
///         Self {
///             sprite: Sprite2D::from_app(app)
///                 .with_model(|m| m.size = Vec2::ONE * 0.2)
///                 .with_model(|m| m.camera = camera)
///         }
///     }
///
///     fn update(&mut self, app: &mut App) {
///         self.sprite.update(app);
///     }
/// }
///
/// struct MovingCamera {
///     camera: Camera2D
/// }
///
/// impl FromApp for MovingCamera {
///     fn from_app(app: &mut App) -> Self {
///         let target = app.get_mut::<Window>().target.to_ref();
///         Self {
///             camera: Camera2D::new(app, vec![target])
///                 .with_size(Vec2::ONE * 0.5) // zoom x2
///         }
///     }
/// }
///
/// impl State for MovingCamera {
///     fn update(&mut self, app: &mut App) {
///         self.camera.position += Vec2::new(0.1, 0.2);
///         self.camera.update(app);
///     }
/// }
/// ```
#[derive(Debug, Builder)]
pub struct Camera2D {
    #[builder(form(value))]
    /// Position of the rendered zone center in world units.
    pub position: Vec2,
    /// Size of the rendered zone in world units.
    #[builder(form(value))]
    pub size: Vec2,
    /// Rotation in radians of the camera around its [`position`](#structfield.position).
    #[builder(form(value))]
    pub rotation: f32,
    /// The render targets where the camera should be used.
    ///
    /// If a camera is linked to a target, then all models linked to the camera are rendered in the
    /// target.
    #[builder(form(closure))]
    pub targets: Vec<GlobRef<Target>>,
    glob: Glob<Camera2DGlob>,
}

impl Camera2D {
    /// Creates a new camera.
    pub fn new(app: &mut App, targets: Vec<GlobRef<Target>>) -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            targets,
            glob: Glob::from_app(app),
        }
    }

    /// Updates the camera.
    pub fn update(&mut self, app: &mut App) {
        let target_sizes = self.target_sizes(app);
        let gpu = app.get_mut::<GpuManager>().get_or_init().clone();
        let glob = self.glob.get_mut(app);
        glob.position = self.position;
        glob.size = self.size;
        glob.rotation = self.rotation;
        glob.register_targets(&self.targets);
        for (target_index, target_size) in target_sizes {
            let transform = self.gpu_transform(target_size.into());
            glob.update_target(&gpu, target_index, transform);
        }
    }

    /// Returns a reference to global data.
    pub fn glob(&self) -> &Glob<Camera2DGlob> {
        &self.glob
    }

    fn gpu_transform(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let position = Vec3::new(-self.position.x, -self.position.y, -1.);
        let scale = Vec3::new(2. * x_scale / self.size.x, 2. * y_scale / self.size.y, -1.);
        Mat4::from_position(position)
            * Quat::from_z(self.rotation).matrix()
            * Mat4::from_scale(scale)
    }

    fn target_sizes(&self, app: &App) -> Vec<(usize, Size)> {
        self.targets
            .iter()
            .map(|target| (target.index(), target.get(app).size()))
            .collect()
    }
}

/// The global data of a [`Camera2D`].
#[derive(Debug, Global)]
pub struct Camera2DGlob {
    pub(crate) position: Vec2,
    pub(crate) size: Vec2,
    pub(crate) rotation: f32,
    pub(crate) targets: Vec<GlobRef<Target>>,
    target_uniforms: FxHashMap<usize, CameraUniform>,
}

impl Default for Camera2DGlob {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            size: Vec2::ONE,
            rotation: 0.,
            targets: vec![],
            target_uniforms: FxHashMap::default(),
        }
    }
}

impl Camera2DGlob {
    /// Converts a `target_position` for a target surface of size `target_size` into world position.
    ///
    /// `target_size` and `target_position` are expressed in pixels.
    /// `target_position` with a value of [`Vec2::ZERO`] corresponds to the top-left
    /// corner of the surface.
    pub fn world_position(&self, target_size: Size, target_position: Vec2) -> Vec2 {
        let target_size = target_size.into();
        self.world_transform(target_size)
            * Vec2::new(
                target_position.x / target_size.x - 0.5,
                0.5 - target_position.y / target_size.y,
            )
    }

    pub(crate) fn bind_group(&self, target_index: usize) -> Option<&BindGroup> {
        self.target_uniforms
            .get(&target_index)
            .map(|uniform| &uniform.bind_group.inner)
    }

    fn register_targets(&mut self, targets: &[GlobRef<Target>]) {
        let target_indexes: Vec<_> = targets.iter().map(|target| target.index()).collect();
        self.target_uniforms
            .retain(|target_index, _| target_indexes.contains(target_index));
        self.targets = targets.into();
    }

    fn update_target(&mut self, gpu: &Gpu, target_index: usize, transform: Mat4) {
        match self.target_uniforms.entry(target_index) {
            Entry::Occupied(mut entry) => entry.get_mut().update(gpu, transform),
            Entry::Vacant(entry) => {
                entry.insert(CameraUniform::new(gpu, transform));
            }
        }
    }

    fn world_transform(&self, target_size: Vec2) -> Mat4 {
        let x_scale = 1.0_f32.min(target_size.y / target_size.x);
        let y_scale = 1.0_f32.min(target_size.x / target_size.y);
        let scale = self.size.with_scale(Vec2::new(1. / x_scale, 1. / y_scale));
        Mat4::from_scale(scale.with_z(1.))
            * Quat::from_z(-self.rotation).matrix()
            * Mat4::from_position(self.position.with_z(0.))
    }
}

#[derive(Debug)]
struct CameraUniform {
    bind_group: BufferBindGroup,
    buffer: Buffer<[[f32; 4]; 4]>,
    transform: Mat4,
}

impl CameraUniform {
    const BINDING: u32 = 0;

    fn new(gpu: &Gpu, transform: Mat4) -> Self {
        let buffer = Buffer::new(
            gpu,
            &[transform.to_array()],
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            "camera_2d",
        );
        Self {
            bind_group: BufferBindGroup::uniform(
                gpu,
                &buffer,
                Self::BINDING,
                &gpu.camera_bind_group_layout,
                "camera_2d",
            ),
            buffer,
            transform,
        }
    }

    fn update(&mut self, gpu: &Gpu, transform: Mat4) {
        if transform != self.transform {
            self.buffer.update(gpu, &[transform.to_array()]);
            self.transform = transform;
        }
    }
}
