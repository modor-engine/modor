use crate::{
    Camera2D, InstanceGroup2D, InstanceRendering2D, Material, MaterialSource, MaterialSync,
};
use modor::{BuiltEntity, ComponentSystems, EntityBuilder, QueryEntityFilter, QueryFilter};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource};

// TODO: update doc (always use high level methods)

/// Creates a 2D instance group entity.
///
/// The created entity contains the following components:
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - All component created by [`material`] if `material` is [`None`]
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
///     instance_group_2d::<Default2DMaterial, With<RedRectangle>>(WINDOW_CAMERA_2D, None)
///         .updated(|m: &mut Default2DMaterial| m.color = Color::RED)
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
pub fn instance_group_2d<M, F>(
    camera_key: ResKey<Camera2D>,
    material_key: Option<ResKey<Material>>,
) -> impl BuiltEntity
where
    M: ComponentSystems + MaterialSource + Default,
    F: QueryEntityFilter,
{
    let filter = QueryFilter::new::<F>();
    let group_key = ResKey::unique("instance-group-2d(modor_graphics)");
    let group = InstanceGroup2D::from_filter(group_key, filter);
    rendered_instance_group_2d::<M>(camera_key, material_key, group)
}

/// Creates a 2D instance entity.
///
/// The created entity contains the following components:
/// - [`Transform2D`]
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - All component created by [`material`] if `material` is [`None`]
///
/// This method is useful to quickly create a rendered entity.
/// However, for performance reasons, consider using [`instance_group_2d`] instead if multiple
/// entities are rendered with the same camera, material and mesh.
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
///     instance_2d::<Default2DMaterial>(WINDOW_CAMERA_2D, None)
///         .updated(|t: &mut Transform2D| t.size = Vec2::new(0.2, 0.1))
///         .updated(|m: &mut Default2DMaterial| m.color = Color::RED)
/// }
/// ```
pub fn instance_2d<M>(
    camera_key: ResKey<Camera2D>,
    material_key: Option<ResKey<Material>>,
) -> impl BuiltEntity
where
    M: ComponentSystems + MaterialSource + Default,
{
    let group_key = ResKey::unique("instance-2d(modor_graphics)");
    let group = InstanceGroup2D::from_self(group_key);
    rendered_instance_group_2d::<M>(camera_key, material_key, group).component(Transform2D::new())
}

fn rendered_instance_group_2d<M>(
    camera_key: ResKey<Camera2D>,
    material_key: Option<ResKey<Material>>,
    group: InstanceGroup2D,
) -> impl BuiltEntity
where
    M: ComponentSystems + MaterialSource + Default,
{
    let is_material_missing = material_key.is_none();
    let material_key =
        material_key.unwrap_or_else(|| ResKey::unique("instance-group-2d(modor_graphics)"));
    EntityBuilder::new()
        .component_option(is_material_missing.then(|| Material::new(material_key)))
        .component_option(is_material_missing.then(MaterialSync::<M>::default))
        .component_option(is_material_missing.then(M::default))
        .component(InstanceRendering2D::new(
            group.key(),
            camera_key,
            material_key,
        ))
        .component(group)
}
