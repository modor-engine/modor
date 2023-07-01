use crate::components::mesh::{Mesh, RECTANGLE_MESH};
use crate::{Camera2D, Material};
use modor_resources::ResKey;

/// A rendered model.
///
/// # Requirements
///
/// Model is rendered only if:
/// - graphics [`module`](crate::module()) is initialized
/// - [`Transform2D`](modor_physics::Transform2D) component is in the same entity
///
/// # Related components
///
/// - [`Transform2D`](modor_physics::Transform2D)
/// - [`ZIndex2D`](crate::ZIndex2D)
/// - [`Material`](Material)
/// - [`Camera2D`](Camera2D)
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
/// const RED_RECTANGLE_MATERIAL: ResKey<Material> = ResKey::new("red-rectangle");
/// const GREEN_ELLIPSE_MATERIAL: ResKey<Material> = ResKey::new("green-ellipse");
/// const CAMERA: ResKey<Camera2D> = ResKey::new("main");
///
/// fn root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .child_entity(Material::new(RED_RECTANGLE_MATERIAL).with_color(Color::RED))
///         .child_entity(Material::ellipse(GREEN_ELLIPSE_MATERIAL).with_color(Color::GREEN))
///         .child_entity(red_rectangle(Vec2::ZERO, Vec2::new(0.5, 0.2)))
///         .child_entity(green_ellipse(Vec2::new(-0.25, 0.25), Vec2::new(0.1, 0.1)))
/// }
///
/// fn red_rectangle(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new().with_position(position).with_size(size))
///         .component(Model::rectangle(RED_RECTANGLE_MATERIAL, CAMERA))
/// }
///
/// fn green_ellipse(position: Vec2, size: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new().with_position(position).with_size(size))
///         .component(Model::rectangle(GREEN_ELLIPSE_MATERIAL, CAMERA))
/// }
/// ```
#[must_use]
#[derive(Component, NoSystem)]
pub struct Model {
    /// Key of the [`Material`](Material) used to render the model.
    ///
    /// If this material does not exist or has a not loaded texture attached,
    /// then the model is not rendered.
    pub material_key: ResKey<Material>,
    /// Keys of the [`Camera2D`](Camera2D)s that must render the model.
    ///
    /// Default is no camera.
    pub camera_keys: Vec<ResKey<Camera2D>>,
    pub(crate) mesh_key: ResKey<Mesh>,
}

impl Model {
    /// Creates a new model from a rectangle mesh with a unique
    /// [`material_key`](#structfield.material_key) and linked to a [`Camera2D`](Camera2D).
    pub fn rectangle(material_key: ResKey<Material>, camera_key: ResKey<Camera2D>) -> Self {
        Self {
            material_key,
            camera_keys: vec![camera_key],
            mesh_key: RECTANGLE_MESH,
        }
    }

    /// Creates a new model from a rectangle mesh with a unique
    /// [`material_key`](#structfield.material_key) and not linked to a [`Camera2D`](Camera2D).
    pub fn hidden_rectangle(material_key: ResKey<Material>) -> Self {
        Self {
            material_key,
            camera_keys: vec![],
            mesh_key: RECTANGLE_MESH,
        }
    }

    /// Returns the model with a new `key` added to the [`camera_keys`](#structfield.camera_keys).
    pub fn with_camera_key(mut self, key: ResKey<Camera2D>) -> Self {
        self.camera_keys.push(key);
        self
    }
}
