use crate::components::mesh::MeshKey;
use modor_resources::{IntoResourceKey, ResourceKey};

/// A rendered model.
///
/// The entity also needs a [`Transform2D`](modor_physics::Transform2D) to define how the model
/// is rendered.
///
/// [`module`](crate::module()) needs to be initialized.
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
///     EntityBuilder::new()
///         .with_child(Material::new(MaterialKey::RedRectangle).with_color(Color::RED))
///         .with_child(Material::ellipse(MaterialKey::GreenEllipse).with_color(Color::GREEN))
///         .with_child(red_rectangle(Vec2::ZERO, Vec2::new(0.5, 0.2)))
///         .with_child(green_ellipse(Vec2::new(-0.25, 0.25), Vec2::new(0.1, 0.1)))
/// }
///
/// fn red_rectangle(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new().with_position(position).with_size(size))
///         .with(Model::rectangle(MaterialKey::RedRectangle).with_camera_key(CameraKey))
/// }
///
/// fn green_ellipse(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Transform2D::new().with_position(position).with_size(size))
///         .with(Model::rectangle(MaterialKey::GreenEllipse).with_camera_key(CameraKey))
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// enum MaterialKey {
///     RedRectangle,
///     GreenEllipse,
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// struct CameraKey;
/// ```
#[must_use]
#[derive(Component, NoSystem)]
pub struct Model {
    /// Key of the [`Material`](crate::Material) used to render the model.
    ///
    /// If this material does not exist or has a not loaded texture attached,
    /// then the model is not rendered.
    pub material_key: ResourceKey,
    /// Keys of the [`Camera2D`](crate::Camera2D)s that must render the model.
    ///
    /// Default is no camera.
    pub camera_keys: Vec<ResourceKey>,
    pub(crate) mesh_key: ResourceKey,
}

impl Model {
    /// Creates a new model from a rectangle mesh and a unique
    /// [`material_key`](#structfield.material_key).
    pub fn rectangle(material_key: impl IntoResourceKey) -> Self {
        Self {
            material_key: material_key.into_key(),
            camera_keys: vec![],
            mesh_key: MeshKey::Rectangle.into_key(),
        }
    }

    /// Returns the model with a new `key` added to the [`camera_keys`](#structfield.camera_keys).
    pub fn with_camera_key(mut self, key: impl IntoResourceKey) -> Self {
        self.camera_keys.push(key.into_key());
        self
    }
}
