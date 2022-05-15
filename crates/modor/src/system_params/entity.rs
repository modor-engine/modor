use crate::entity::internal::{EntityGuard, EntityGuardBorrow, EntityIter};
use crate::storages::archetypes::EntityLocation;
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityIdx;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{QuerySystemParamWithLifetime, SystemParamWithLifetime};
use crate::{QuerySystemParam, SystemData, SystemInfo, SystemParam};
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
    pub(crate) data: SystemData<'a>,
}

impl<'a> Entity<'a> {
    /// Returns the entity ID.
    ///
    /// Entity IDs are unique and can be recycled in case the entity is deleted.
    pub fn id(self) -> usize {
        self.entity_idx.into()
    }

    /// Returns the entity parent.
    pub fn parent(self) -> Option<Self> {
        self.data
            .entities
            .parent_idx(self.entity_idx)
            .map(|p| Self {
                entity_idx: p,
                data: self.data,
            })
    }

    /// Returns an iterator on entity children.
    pub fn children(
        &self,
    ) -> impl Iterator<Item = Entity<'_>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.data
            .entities
            .child_idxs(self.entity_idx)
            .iter()
            .map(|&c| Self {
                entity_idx: c,
                data: self.data,
            })
    }

    /// Returns the entity depth in the entity hierarchy.
    ///
    /// Root entities have a depth of `0`.
    pub fn depth(self) -> usize {
        self.data.entities.depth(self.entity_idx)
    }
}

impl<'a> SystemParamWithLifetime<'a> for Entity<'_> {
    type Param = Entity<'a>;
    type Guard = EntityGuard<'a>;
    type GuardBorrow = EntityGuardBorrow<'a>;
    type Stream = EntityIter<'a>;
}

impl SystemParam for Entity<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: false,
            filtered_component_type_idxs: vec![],
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        EntityGuard::new(data, info)
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
    fn filtered_component_type_idxs(_data: SystemData<'_>) -> Vec<ComponentTypeIdx> {
        vec![]
    }

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
            .data
            .archetypes
            .entity_idxs(location.idx)
            .get(location.pos)
            .map(|&e| Entity {
                entity_idx: e,
                data: guard.data,
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

mod internal {
    use crate::storages::archetypes::FilteredArchetypeIdxIter;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, SystemData, SystemInfo};
    use std::iter::Flatten;
    use std::slice::Iter;

    pub struct EntityGuard<'a> {
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a> EntityGuard<'a> {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self { data, info }
        }

        pub(crate) fn borrow(&mut self) -> EntityGuardBorrow<'_> {
            EntityGuardBorrow {
                item_count: self.info.item_count,
                sorted_archetype_idxs: self
                    .data
                    .filter_archetype_idx_iter(self.info.filtered_component_type_idxs),
                data: self.data,
            }
        }
    }

    pub struct EntityGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct EntityIter<'a> {
        entity_idxs: Flatten<ArchetypeEntityIdxIter<'a>>,
        len: usize,
        data: SystemData<'a>,
    }

    impl<'a> EntityIter<'a> {
        pub fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                entity_idxs: ArchetypeEntityIdxIter::new(guard).flatten(),
                len: guard.item_count,
                data: guard.data,
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
                    data: self.data,
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
                    data: self.data,
                }
            })
        }
    }

    impl ExactSizeIterator for EntityIter<'_> {}

    struct ArchetypeEntityIdxIter<'a> {
        sorted_archetype_idxs: FilteredArchetypeIdxIter<'a>,
        data: SystemData<'a>,
    }

    impl<'a> ArchetypeEntityIdxIter<'a> {
        fn new(guard: &'a EntityGuardBorrow<'_>) -> Self {
            Self {
                sorted_archetype_idxs: guard.sorted_archetype_idxs.clone(),
                data: guard.data,
            }
        }
    }

    impl<'a> Iterator for ArchetypeEntityIdxIter<'a> {
        type Item = Iter<'a, EntityIdx>;

        fn next(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next()
                .map(|a| self.data.archetypes.entity_idxs(a).iter())
        }
    }

    impl DoubleEndedIterator for ArchetypeEntityIdxIter<'_> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.sorted_archetype_idxs
                .next_back()
                .map(|a| self.data.archetypes.entity_idxs(a).iter())
        }
    }
}
