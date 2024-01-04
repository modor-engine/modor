use crate::{Material, MaterialSource, MaterialSync};
use modor::{BuiltEntity, EntityBuilder};
use modor_resources::ResKey;

/// Creates a material entity.
///
/// The created entity contains the following components:
/// - [`Material`]
/// - [`MaterialSync<M>`]
/// - Component of type `M`
///
/// # Entity functions creating this entity
///
/// - [`instance_group_2d`](crate::instance_group_2d())
/// - [`instance_2d`](crate::instance_2d())
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// const BLUE_ELLIPSE_MATERIAL: ResKey<Material> = ResKey::new("blue-ellipse");
///
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(window_target())
///     .with_entity(blue_ellipse_material())
///     .with_entity(blue_ellipse())
///     .run(modor_graphics::runner);
/// # }
///
/// fn blue_ellipse_material() -> impl BuiltEntity {
///     material(BLUE_ELLIPSE_MATERIAL, Default2DMaterial::default())
///         .updated(|m: &mut Default2DMaterial| m.is_ellipse = true)
///         .updated(|m: &mut Default2DMaterial| m.color = Color::BLUE)
/// }
///
/// fn blue_ellipse() -> impl BuiltEntity {
///     instance_2d(WINDOW_CAMERA_2D, BLUE_ELLIPSE_MATERIAL)
/// }
/// ```
pub fn material<M>(key: ResKey<Material>, material: M) -> impl BuiltEntity
where
    M: MaterialSource,
{
    EntityBuilder::new()
        .component(Material::new(key))
        .component(MaterialSync::<M>::default())
        .component(material)
}
