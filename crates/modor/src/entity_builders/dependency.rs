use crate::entity_builders::internal::BuiltEntityPart;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::{BuiltEntity, Component, True};
use std::any;
use std::any::TypeId;
use std::marker::PhantomData;

/// A builder for defining dependency of an entity.
///
/// [`EntityBuilder`](crate::EntityBuilder) needs to be used to instantiate this builder.
pub struct EntityDependencyBuilder<C, E, F> {
    pub(crate) builder: F,
    pub(crate) phantom: PhantomData<fn(C) -> E>,
}

impl<C, E, F> BuiltEntityPart for EntityDependencyBuilder<C, E, F>
where
    C: Component<IsSingleton = True>,
    E: BuiltEntity,
    F: FnOnce() -> E,
{
    fn create_other_entities(self, core: &mut CoreStorage, _parent_idx: Option<EntityIdx>) {
        let singleton_exists = core
            .components()
            .type_idx(TypeId::of::<C>())
            .and_then(|c| core.components().singleton_location(c))
            .is_some();
        if singleton_exists {
            trace!(
                "dependency entity for singleton of type `{}` not created as already existing", // no-coverage
                any::type_name::<F>(), // no-coverage
            );
        } else {
            (self.builder)().build(core, None);
            trace!(
                "dependency entity for singleton of type `{}` created", // no-coverage
                any::type_name::<F>(),                                  // no-coverage
            );
        }
    }
}
