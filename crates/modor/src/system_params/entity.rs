use crate::entity::internal::{EntityGuard, EntityGuardBorrow, EntityIter};
use crate::storages::archetypes::EntityLocation;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::systems::context::SystemContext;
use crate::{QuerySystemParam, SystemParam};
use std::iter::FusedIterator;

/// A system parameter for retrieving information about the entity.
///
/// # Examples
///
/// ```rust
/// # use modor::Entity;
/// #
/// #[derive(Debug)]
/// struct Position(f32, f32);
///
/// fn print_position(position: &Position, entity: Entity<'_>) {
///     println!("Entity with ID {} has position {:?}", entity.id(), position)
/// }
/// ```
#[derive(Clone, Copy)]
pub struct Entity<'a> {
    pub(crate) entity_idx: EntityIdx,
    pub(crate) context: SystemContext<'a>,
}

impl<'a> Entity<'a> {
    /// Returns the entity ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    #[must_use]
    pub fn id(self) -> usize {
        self.entity_idx.into()
    }

    /// Returns the entity parent.
    #[must_use]
    pub fn parent(self) -> Option<Self> {
        self.context
            .storages
            .entities
            .parent_idx(self.entity_idx)
            .map(|p| Self {
                entity_idx: p,
                context: self.context,
            })
    }

    /// Returns an iterator on entity children.
    pub fn children<'b>(
        &'b self,
    ) -> impl Iterator<Item = Entity<'a>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator + 'b
    {
        self.context
            .storages
            .entities
            .child_idxs(self.entity_idx)
            .iter()
            .map(|&c| Self {
                entity_idx: c,
                context: self.context,
            })
    }

    /// Returns the entity depth in the entity hierarchy.
    ///
    /// Root entities have a depth of `0`.
    #[must_use]
    pub fn depth(self) -> usize {
        self.context.storages.entities.depth(self.entity_idx)
    }
}

impl<'a> SystemParamWithLifetime<'a> for Entity<'_> {
    type Param = Entity<'a>;
    type Guard = EntityGuard<'a>;
    type GuardBorrow = EntityGuardBorrow<'a>;
    type Stream = EntityIter<'a>;
}

impl SystemParam for Entity<'_> {
    type Filter = ();
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: false,
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        EntityGuard::new(context)
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        guard.borrow()
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        EntityIter::new(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream.next()
    }
}

impl<'a> QuerySystemParamWithLifetime<'a> for Entity<'_> {
    type ConstParam = Entity<'a>;
    type Iter = EntityIter<'a>;
    type IterMut = EntityIter<'a>;
}

impl QuerySystemParam for Entity<'_> {
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a,
    {
        EntityIter::new(guard)
    }

    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a,
    {
        EntityIter::new(guard)
    }

    #[inline]
    fn get<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as QuerySystemParamWithLifetime<'a>>::ConstParam>
    where
        'b: 'a,
    {
        guard
            .context
            .storages
            .archetypes
            .entity_idxs(location.idx)
            .get(location.pos)
            .map(|&e| Entity {
                entity_idx: e,
                context: guard.context,
            })
    }

    #[inline]
    fn get_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location: EntityLocation,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        Self::get(guard, location)
    }

    #[inline]
    fn get_both_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        location1: EntityLocation,
        location2: EntityLocation,
    ) -> (
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
        Option<<Self as SystemParamWithLifetime<'a>>::Param>,
    )
    where
        'b: 'a,
    {
        (Self::get(guard, location1), Self::get(guard, location2))
    }
}

pub(super) mod internal {
    use crate::storages::entities::EntityIdx;
    use crate::systems::context::SystemContext;
    use crate::systems::iterations::FilteredArchetypeIdxIter;
    use crate::Entity;
    use std::iter::Flatten;
    use std::slice::Iter;

    pub struct EntityGuard<'a> {
        context: SystemContext<'a>,
    }

    impl<'a> EntityGuard<'a> {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self { context }
        }

        pub(crate) fn borrow(&mut self) -> EntityGuardBorrow<'_> {
            EntityGuardBorrow {
                item_count: self.context.item_count,
                sorted_archetype_idxs: self.context.filter_archetype_idx_iter(),
                context: self.context,
            }
        }
    }

    pub struct EntityGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct EntityIter<'a> {
        entity_idxs: Flatten<ArchetypeEntityIdxIter<'a>>,
        len: usize,
        context: SystemContext<'a>,
    }

    impl<'a> EntityIter<'a> {
        pub fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                entity_idxs: ArchetypeEntityIdxIter::new(guard).flatten(),
                len: guard.item_count,
                context: guard.context,
            }
        }
    }

    impl<'a> Iterator for EntityIter<'a> {
        type Item = Entity<'a>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item> {
            self.entity_idxs.next().map(|&e| {
                self.len -= 1;
                Entity {
                    entity_idx: e,
                    context: self.context,
                }
            })
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.len, Some(self.len))
        }
    }

    impl DoubleEndedIterator for EntityIter<'_> {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item> {
            self.entity_idxs.next_back().map(|&e| {
                self.len -= 1;
                Entity {
                    entity_idx: e,
                    context: self.context,
                }
            })
        }
    }

    impl ExactSizeIterator for EntityIter<'_> {}

    struct ArchetypeEntityIdxIter<'a> {
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        context: SystemContext<'a>,
    }

    impl<'a> ArchetypeEntityIdxIter<'a> {
        fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                context: guard.context,
            }
        }
    }

    impl<'a> Iterator for ArchetypeEntityIdxIter<'a> {
        type Item = Iter<'a, EntityIdx>;

        fn next(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next()
                .map(|a| self.context.storages.archetypes.entity_idxs(a).iter())
        }
    }

    impl DoubleEndedIterator for ArchetypeEntityIdxIter<'_> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next_back()
                .map(|a| self.context.storages.archetypes.entity_idxs(a).iter())
        }
    }
}
