use crate::{Camera2D, Material, Model};
use modor::{BuiltEntity, EntityBuilder};
use modor_physics::Transform2D;
use modor_resources::ResKey;

/// Creates a 2D model.
///
/// The created entity contains the following components:
/// - [`Transform2D`]
/// - [`Model`]
/// - [`Material`] if `material` is not [`Model2DMaterial::Key`]
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(window_target())
///     .with_entity(red_rectangle())
///     .run(modor_graphics::runner);
/// # }
///
/// fn red_rectangle() -> impl BuiltEntity {
///     model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
///         .updated(|t: &mut Transform2D| *t.size = Vec2::new(0.2, 0.1))
///         .updated(|m: &mut Material| m.color = Color::RED)
/// }
/// ```
pub fn model_2d(camera_key: ResKey<Camera2D>, material: Model2DMaterial) -> impl BuiltEntity {
    let material_key = match material {
        Model2DMaterial::Key(key) => key,
        Model2DMaterial::Rectangle | Model2DMaterial::Ellipse => {
            ResKey::unique("model-2d(modor_graphics)")
        }
    };
    let material = match material {
        Model2DMaterial::Key(_) => None,
        Model2DMaterial::Rectangle => Some(Material::new(material_key)),
        Model2DMaterial::Ellipse => Some(Material::ellipse(material_key)),
    };
    EntityBuilder::new()
        .component(Transform2D::new())
        .component(Model::rectangle(material_key, camera_key))
        .component_option(material)
}

/// The material attached to a model created with [`model_2d`].
///
/// # Examples
///
/// See [`model_2d`].
pub enum Model2DMaterial {
    /// Existing material.
    Key(ResKey<Material>),
    /// New white rectangle material specific to the model.
    Rectangle,
    /// New white ellipse material specific to the model.
    Ellipse,
}
