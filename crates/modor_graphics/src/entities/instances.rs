use crate::{Camera2D, InstanceGroup2D, InstanceRendering2D, Material};
use modor::{BuiltEntity, EntityBuilder, QueryEntityFilter, QueryFilter};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource};

/// Creates a 2D instance group entity.
///
/// The created entity contains the following components:
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - [`Material`] if `material` is not [`MaterialType::Key`]
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
///     .with_entity(red_rectangle_instance_group())
///     .with_entity(red_rectangle(Vec2::new(0.25, -0.25)))
///     .with_entity(red_rectangle(Vec2::new(-0.3, 0.1)))
///     .run(modor_graphics::runner);
/// # }
///
/// fn red_rectangle_instance_group() -> impl BuiltEntity {
///     instance_group_2d::<With<RedRectangle>>(WINDOW_CAMERA_2D, MaterialType::Rectangle)
///         .updated(|m: &mut Material| m.color = Color::RED)
/// }
///
/// fn red_rectangle(position: Vec2) -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Transform2D::new())
///         .with(|t| t.position = position)
///         .with(|t| t.size = Vec2::new(0.2, 0.1))
///         .component(RedRectangle)
/// }
///
/// #[derive(Component, NoSystem)]
/// struct RedRectangle;
/// ```
pub fn instance_group_2d<F>(
    camera_key: ResKey<Camera2D>,
    material: MaterialType,
) -> impl BuiltEntity
where
    F: QueryEntityFilter,
{
    let filter = QueryFilter::new::<F>();
    let group_key = ResKey::unique("instance-group-2d(modor_graphics)");
    let group = InstanceGroup2D::from_filter(group_key, filter);
    rendered_instance_group_2d(camera_key, material, group)
}

/// Creates a 2D instance entity.
///
/// The created entity contains the following components:
/// - [`Transform2D`]
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - [`Material`] if `material` is not [`MaterialType::Key`]
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
///     instance_2d(WINDOW_CAMERA_2D, MaterialType::Rectangle)
///         .updated(|t: &mut Transform2D| t.size = Vec2::new(0.2, 0.1))
///         .updated(|m: &mut Material| m.color = Color::RED)
/// }
/// ```
pub fn instance_2d(camera_key: ResKey<Camera2D>, material: MaterialType) -> impl BuiltEntity {
    let group_key = ResKey::unique("instance-2d(modor_graphics)");
    let group = InstanceGroup2D::from_self(group_key);
    rendered_instance_group_2d(camera_key, material, group).component(Transform2D::new())
}

fn rendered_instance_group_2d(
    camera_key: ResKey<Camera2D>,
    material: MaterialType,
    group: InstanceGroup2D,
) -> impl BuiltEntity {
    let material_key = match material {
        MaterialType::Key(key) => key,
        MaterialType::Rectangle | MaterialType::Ellipse => {
            ResKey::unique("instance-group-2d(modor_graphics)")
        }
    };
    let material = match material {
        MaterialType::Key(_) => None,
        MaterialType::Rectangle => Some(Material::new(material_key)),
        MaterialType::Ellipse => Some(Material::ellipse(material_key)),
    };
    EntityBuilder::new()
        .component_option(material)
        .component(InstanceRendering2D::new(
            group.key(),
            camera_key,
            material_key,
        ))
        .component(group)
}

/// The type of material attached to a model created with [`instance_2d`].
///
/// # Examples
///
/// See [`instance_2d`].
pub enum MaterialType {
    /// Existing material.
    Key(ResKey<Material>),
    /// New white rectangle material specific to the model.
    Rectangle,
    /// New white ellipse material specific to the model.
    Ellipse,
}
