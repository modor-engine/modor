use crate::instances::ResourceKeys;
use crate::keys::cameras::{CameraKey, DefaultCameraRef};
use crate::keys::models::{ModelKey, ModelRef};
use crate::keys::shaders::{ShaderKey, ShaderRef};
use crate::{CameraRef, Color};

#[derive(Clone, Debug, Component)]
pub struct Mesh2D {
    /// Color of the entity.
    ///
    /// This color will be applied only if there is no texture or if the texture is not loaded.
    ///
    /// Some optimizations are perform on shapes with alpha component equal to zero.
    pub color: Color,
    /// Z-coordinate of the mesh used to define display order, where smallest Z-coordinates are
    /// displayed first.
    pub z: f32,
    pub(crate) resource_keys: ResourceKeys,
}

impl Mesh2D {
    /// Creates a new white rectangle.
    ///
    /// The rectangle size is driven by the [`Transform2D`](modor_physics::Transform2D) size along
    /// X-axis and Y-axis.
    #[must_use]
    pub fn rectangle() -> Self {
        Self::new(ShaderRef::Rectangle, ModelRef::Rectangle)
    }

    /// Creates a new white ellipse.
    ///
    /// The ellipse major and minor radii are driven by the
    /// [`Transform2D`](modor_physics::Transform2D) size along X-axis and Y-axis.
    #[must_use]
    pub fn ellipse() -> Self {
        Self::new(ShaderRef::Ellipse, ModelRef::Rectangle)
    }

    /// Returns the mesh with a different `color`.
    ///
    /// Default value is `Color::WHITE`.
    #[must_use]
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Returns the mesh with a different `z`.
    ///
    /// Default value is `0.0`.
    #[must_use]
    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Returns the mesh with an attached camera.
    ///
    /// Default attached camera has size `Vec2::new(1., 1.)` and center `Vec2::new(0., 0.)`.
    #[must_use]
    pub fn with_camera(mut self, ref_: impl CameraRef) -> Self {
        self.resource_keys.camera = CameraKey::new(ref_);
        self
    }

    /// Attach a new camera.
    pub fn attach_camera(&mut self, ref_: impl CameraRef) {
        self.resource_keys.camera = CameraKey::new(ref_);
    }

    /// Attach the default camera with size `Vec2::new(1., 1.)` and center `Vec2::new(0., 0.)`.
    pub fn attach_default_camera(&mut self) {
        self.resource_keys.camera = CameraKey::new(DefaultCameraRef);
    }

    fn new(shader_ref: ShaderRef, model_ref: ModelRef) -> Self {
        Self {
            color: Color::WHITE,
            z: 0.,
            resource_keys: ResourceKeys {
                shader: ShaderKey::new(shader_ref),
                model: ModelKey::new(model_ref),
                camera: CameraKey::new(DefaultCameraRef),
            },
        }
    }
}
