use crate::entities::instance::private::MaterialRefValue;
use crate::{
    Camera2D, Default2DMaterial, InstanceGroup2D, InstanceRendering2D, Material, MaterialSource,
    MaterialSync,
};
use modor::{BuiltEntity, ComponentSystems, EntityBuilder, QueryEntityFilter, QueryFilter};
use modor_physics::Transform2D;
use modor_resources::{ResKey, Resource};

/// Creates a 2D instance group entity.
///
/// The created entity contains the following components:
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - All component created by [`material`](crate::material()) if `material_ref` refers to a new
///     material.
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
///     instance_group_2d::<With<RedRectangle>>(WINDOW_CAMERA_2D, Default2DMaterial::new())
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
///
/// See [`material`](crate::material) to specify an existing material.
pub fn instance_group_2d<F>(
    camera_key: ResKey<Camera2D>,
    material_ref: impl MaterialRef,
) -> impl BuiltEntity
where
    F: QueryEntityFilter,
{
    let filter = QueryFilter::new::<F>();
    let group_key = ResKey::unique("instance-group-2d(modor_graphics)");
    let group = InstanceGroup2D::from_filter(group_key, filter);
    rendered_instance_group_2d(camera_key, material_ref, group)
}

/// Creates a 2D instance entity.
///
/// The created entity contains the following components:
/// - [`Transform2D`]
/// - [`InstanceGroup2D`]
/// - [`InstanceRendering2D`]
/// - All component created by [`material`](crate::material()) if `material_ref` refers to a new
///     material.
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
///     instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
///         .updated(|t: &mut Transform2D| t.size = Vec2::new(0.2, 0.1))
///         .updated(|m: &mut Default2DMaterial| m.color = Color::RED)
/// }
/// ```
///
/// See [`material`](crate::material) to specify an existing material.
pub fn instance_2d(
    camera_key: ResKey<Camera2D>,
    material_ref: impl MaterialRef,
) -> impl BuiltEntity {
    let group_key = ResKey::unique("instance-2d(modor_graphics)");
    let group = InstanceGroup2D::from_self(group_key);
    rendered_instance_group_2d(camera_key, material_ref, group).component(Transform2D::new())
}

fn rendered_instance_group_2d<M>(
    camera_key: ResKey<Camera2D>,
    material_ref: M,
    group: InstanceGroup2D,
) -> impl BuiltEntity
where
    M: MaterialRef,
{
    let (material_key, material) = match material_ref.value() {
        MaterialRefValue::Key(material_key) => (Some(material_key), None),
        MaterialRefValue::Component(material) => (None, Some(material)),
    };
    let material_key =
        material_key.unwrap_or_else(|| ResKey::unique("instance-group-2d(modor_graphics)"));
    EntityBuilder::new()
        .component_option(material.is_some().then(|| Material::new(material_key)))
        .component_option(
            material
                .is_some()
                .then(MaterialSync::<M::MaterialType>::default),
        )
        .component_option(material)
        .component(InstanceRendering2D::new(
            group.key(),
            camera_key,
            material_key,
        ))
        .component(group)
}

/// A trait implemented for types referencing a [`Material`].
pub trait MaterialRef {
    #[doc(hidden)]
    type MaterialType: ComponentSystems + MaterialSource;

    #[doc(hidden)]
    fn value(self) -> MaterialRefValue<Self::MaterialType>;
}

impl<T> MaterialRef for T
where
    T: ComponentSystems + MaterialSource,
{
    type MaterialType = T;

    fn value(self) -> MaterialRefValue<Self::MaterialType> {
        MaterialRefValue::Component(self)
    }
}

impl MaterialRef for ResKey<Material> {
    type MaterialType = Default2DMaterial;

    fn value(self) -> MaterialRefValue<Self::MaterialType> {
        MaterialRefValue::Key(self)
    }
}

mod private {
    use crate::Material;
    use modor_resources::ResKey;

    pub enum MaterialRefValue<C> {
        Key(ResKey<Material>),
        Component(C),
    }
}
