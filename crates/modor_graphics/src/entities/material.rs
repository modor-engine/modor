use crate::{Material, MaterialSource, MaterialSync};
use modor::{BuiltEntity, ComponentSystems, EntityBuilder};
use modor_resources::ResKey;

/// Creates a material entity.
///
/// The created entity contains the following components:
/// - [`Material`]
/// - [`MaterialSync`]
/// - Component of type `M` created using [`Default`] implementation
///
/// # Examples
///
/// See [`InstanceGroup2D`](crate::InstanceGroup2D).
pub fn material<M>(key: ResKey<Material>) -> impl BuiltEntity
where
    M: ComponentSystems + MaterialSource + Default,
{
    EntityBuilder::new()
        .component(Material::new(key))
        .component(MaterialSync::<M>::default())
        .component(M::default())
}
