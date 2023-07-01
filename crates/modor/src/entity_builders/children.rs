use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::BuiltEntity;

/// A builder for defining children of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityChildrenBuilder<F, P> {
    pub(crate) builder: F,
    pub(crate) previous: P,
}

impl<F, P> BuiltEntityPart for EntityChildrenBuilder<F, P>
where
    F: FnOnce(&mut EntityGenerator<'_>),
    P: BuiltEntity,
{
    fn create_archetype(
        &mut self,
        core: &mut CoreStorage,
        archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        self.previous.create_archetype(core, archetype_idx)
    }

    fn add_components(&mut self, core: &mut CoreStorage, location: EntityLocation) {
        self.previous.add_components(core, location);
    }

    fn create_other_entities(self, core: &mut CoreStorage, parent_idx: Option<EntityIdx>) {
        self.previous.create_other_entities(core, parent_idx);
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
///         .child_entities(|b| {
///             for i in 1..=10 {
///                 b.add(build_child(i));
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
    pub fn add(&mut self, entity: impl BuiltEntity) {
        entity.build(self.core, self.parent_idx);
    }
}
