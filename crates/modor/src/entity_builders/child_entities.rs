use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::BuildableEntity;

/// A builder for defining children of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityChildEntitiesBuilder<F> {
    pub(crate) builder: F,
}

impl<F> BuiltEntityPart for EntityChildEntitiesBuilder<F>
where
    F: FnOnce(&mut EntityGenerator<'_>),
{
    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        (self.builder)(&mut EntityGenerator { core, parent_idx });
    }
}

/// An entity generator.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Value(u32);
///
/// fn build_root() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Value(0))
///         .child_entities(|g| {
///             for i in 1..=10 {
///                 g.add(build_child(i));
///             }
///         })
/// }
///
/// fn build_child(value: u32) -> impl BuiltEntity {
///     EntityBuilder::new().component(Value(value))
/// }
/// ```
pub struct EntityGenerator<'a> {
    core: &'a mut CoreStorage,
    parent_idx: Option<EntityIdx>,
}

impl EntityGenerator<'_> {
    /// Adds a entity.
    pub fn add<T>(&mut self, entity: impl BuildableEntity<T>) {
        entity.build_entity(self.core, self.parent_idx);
    }
}
