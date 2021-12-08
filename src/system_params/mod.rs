use crate::storages::core::SystemProperties;
use crate::system_params::internal::{
    QuerySystemParamWithLifetime, SystemParamIterInfo, SystemParamWithLifetime,
};
use crate::{SystemData, SystemInfo};

pub(crate) mod components;
pub(crate) mod components_mut;
pub(crate) mod entity;
pub(crate) mod optional_components;
pub(crate) mod optional_components_mut;
pub(crate) mod queries;
pub(crate) mod tuples;
pub(crate) mod world;

/// A trait implemented for valid system parameters.
pub trait SystemParam: for<'a> SystemParamWithLifetime<'a> {
    #[doc(hidden)]
    type Tuple: SystemParam;
    #[doc(hidden)]
    type InnerTuple: SystemParam;

    #[doc(hidden)]
    fn properties() -> SystemProperties;

    #[doc(hidden)]
    fn iter_info(data: &SystemData<'_>, info: &SystemInfo) -> SystemParamIterInfo;

    #[doc(hidden)]
    fn lock<'a>(data: &'a SystemData<'_>) -> <Self as SystemParamWithLifetime<'a>>::Guard;

    #[doc(hidden)]
    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a;

    #[doc(hidden)]
    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a;

    #[doc(hidden)]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a;
}

/// A trait implemented for valid [`Query`](crate::Query) parameters.
pub trait QuerySystemParam: SystemParam + for<'a> QuerySystemParamWithLifetime<'a> {
    #[doc(hidden)]
    fn query_iter<'a, 'b>(
        guard: &'a <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::Iter
    where
        'b: 'a;

    #[doc(hidden)]
    fn query_iter_mut<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
        info: &'a SystemParamIterInfo,
    ) -> <Self as QuerySystemParamWithLifetime<'a>>::IterMut
    where
        'b: 'a;
}

pub(crate) mod internal {
    use crate::storages::archetypes::ArchetypeInfo;
    use crate::SystemParam;
    use std::any::Any;

    pub trait SystemParamWithLifetime<'a> {
        type Param: 'a;
        type Guard: 'a;
        type GuardBorrow: 'a;
        type Stream: 'a;
    }

    pub trait QuerySystemParamWithLifetime<'a>: SystemParamWithLifetime<'a> {
        type ConstParam: 'a + SystemParamWithLifetime<'a>;
        type Iter: 'a
            + Sync
            + Send
            + Iterator<Item = <Self::ConstParam as SystemParamWithLifetime<'a>>::Param>
            + DoubleEndedIterator
            + ExactSizeIterator;
        type IterMut: 'a
            + Sync
            + Send
            + Iterator<Item = <Self as SystemParamWithLifetime<'a>>::Param>
            + DoubleEndedIterator
            + ExactSizeIterator;
    }

    pub trait LockableSystemParam: SystemParam {
        type LockedType: Any;
        type Mutability: Mutability;
    }

    pub trait Mutability {}

    pub struct Const;

    impl Mutability for Const {}

    pub struct Mut;

    impl Mutability for Mut {}

    #[derive(Clone, PartialEq, Debug)]
    pub enum SystemParamIterInfo {
        None,
        ComponentUnionEntities(EntityIterInfo),
        ComponentIntersectionEntities(EntityIterInfo),
    }

    impl SystemParamIterInfo {
        pub(crate) fn sorted_archetypes(&self) -> Option<&[ArchetypeInfo]> {
            match self {
                Self::ComponentUnionEntities(EntityIterInfo { sorted_archetypes })
                | Self::ComponentIntersectionEntities(EntityIterInfo { sorted_archetypes }) => {
                    Some(sorted_archetypes)
                }
                Self::None => None,
            }
        }

        pub(crate) fn item_count(&self) -> usize {
            match self {
                Self::None => 1,
                Self::ComponentUnionEntities(EntityIterInfo { sorted_archetypes })
                | Self::ComponentIntersectionEntities(EntityIterInfo { sorted_archetypes }) => {
                    sorted_archetypes.iter().map(|a| a.entity_count).sum()
                }
            }
        }

        pub(crate) fn merge(self, other: Self) -> Self {
            match self {
                Self::None => other,
                Self::ComponentUnionEntities(EntityIterInfo { sorted_archetypes }) => match other {
                    Self::None => {
                        Self::ComponentUnionEntities(EntityIterInfo { sorted_archetypes })
                    }
                    Self::ComponentUnionEntities(EntityIterInfo {
                        sorted_archetypes: other_sorted_archetypes,
                    }) => Self::ComponentUnionEntities(EntityIterInfo {
                        sorted_archetypes: Self::union(sorted_archetypes, other_sorted_archetypes),
                    }),
                    Self::ComponentIntersectionEntities(_) => other,
                },
                Self::ComponentIntersectionEntities(EntityIterInfo { sorted_archetypes }) => {
                    match other {
                        Self::None | Self::ComponentUnionEntities(_) => {
                            Self::ComponentIntersectionEntities(EntityIterInfo {
                                sorted_archetypes,
                            })
                        }
                        Self::ComponentIntersectionEntities(EntityIterInfo {
                            sorted_archetypes: other_sorted_archetypes,
                        }) => Self::ComponentIntersectionEntities(EntityIterInfo {
                            sorted_archetypes: Self::intersection(
                                sorted_archetypes,
                                &other_sorted_archetypes,
                            ),
                        }),
                    }
                }
            }
        }

        fn union(
            mut sorted_archetypes: Vec<ArchetypeInfo>,
            other_sorted_archetypes: Vec<ArchetypeInfo>,
        ) -> Vec<ArchetypeInfo> {
            for archetype in other_sorted_archetypes {
                if let Err(pos) = sorted_archetypes.binary_search(&archetype) {
                    sorted_archetypes.insert(pos, archetype);
                }
            }
            sorted_archetypes
        }

        fn intersection(
            sorted_archetypes: Vec<ArchetypeInfo>,
            other_sorted_archetypes: &[ArchetypeInfo],
        ) -> Vec<ArchetypeInfo> {
            sorted_archetypes
                .into_iter()
                .filter(|a| other_sorted_archetypes.binary_search(a).is_ok())
                .collect()
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct EntityIterInfo {
        pub(crate) sorted_archetypes: Vec<ArchetypeInfo>,
    }
}

#[cfg(test)]
mod system_param_iter_tests {
    use crate::storages::archetypes::{ArchetypeIdx, ArchetypeInfo};
    use crate::system_params::internal::{EntityIterInfo, SystemParamIterInfo};

    impl SystemParamIterInfo {
        pub(crate) fn new_union(archetype_info: Vec<(ArchetypeIdx, usize)>) -> Self {
            Self::ComponentUnionEntities(Self::create_entity_iter_info(archetype_info))
        }

        pub(crate) fn new_intersection(archetype_info: Vec<(ArchetypeIdx, usize)>) -> Self {
            Self::ComponentIntersectionEntities(Self::create_entity_iter_info(archetype_info))
        }

        fn create_entity_iter_info(
            mut archetype_info: Vec<(ArchetypeIdx, usize)>,
        ) -> EntityIterInfo {
            archetype_info.sort_unstable();
            EntityIterInfo {
                sorted_archetypes: archetype_info
                    .into_iter()
                    .map(|(i, c)| ArchetypeInfo {
                        idx: i,
                        entity_count: c,
                    })
                    .collect(),
            }
        }
    }

    #[test]
    fn merge_none_with_none() {
        let iter1 = SystemParamIterInfo::None;
        let iter2 = SystemParamIterInfo::None;

        let result = iter1.clone().merge(iter2.clone());
        let reversed_result = iter1.merge(iter2);

        assert_eq!(result, SystemParamIterInfo::None);
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes(), None);
        assert_eq!(result.item_count(), 1);
    }

    #[test]
    fn merge_none_with_union() {
        let iter1 = SystemParamIterInfo::None;
        let archetypes = vec![create_archetype(2.into(), 15)];
        let iter2 = SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: archetypes.clone(),
        });

        let result = iter2.clone().merge(iter1.clone());
        let reversed_result = iter2.clone().merge(iter1);

        assert_eq!(result, iter2);
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes().unwrap(), archetypes);
        assert_eq!(result.item_count(), 15);
    }

    #[test]
    fn merge_none_with_intersection() {
        let iter1 = SystemParamIterInfo::None;
        let archetypes = vec![create_archetype(2.into(), 15)];
        let iter2 = SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
            sorted_archetypes: archetypes.clone(),
        });

        let result = iter2.clone().merge(iter1.clone());
        let reversed_result = iter2.clone().merge(iter1);

        assert_eq!(result, iter2);
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes().unwrap(), archetypes);
        assert_eq!(result.item_count(), 15);
    }

    #[test]
    fn merge_union_with_union() {
        let iter1 = SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: vec![
                create_archetype(2.into(), 15),
                create_archetype(4.into(), 17),
            ],
        });
        let iter2 = SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: vec![
                create_archetype(2.into(), 15),
                create_archetype(3.into(), 16),
            ],
        });

        let result = iter1.clone().merge(iter2.clone());
        let reversed_result = iter1.merge(iter2);

        let archetypes = vec![
            create_archetype(2.into(), 15),
            create_archetype(3.into(), 16),
            create_archetype(4.into(), 17),
        ];
        assert_eq!(
            result,
            SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
                sorted_archetypes: archetypes.clone(),
            })
        );
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes().unwrap(), archetypes);
        assert_eq!(result.item_count(), 15 + 16 + 17);
    }

    #[test]
    fn merge_union_with_intersection() {
        let iter1 = SystemParamIterInfo::ComponentUnionEntities(EntityIterInfo {
            sorted_archetypes: vec![
                create_archetype(2.into(), 15),
                create_archetype(4.into(), 17),
            ],
        });
        let archetypes = vec![
            create_archetype(2.into(), 15),
            create_archetype(3.into(), 16),
        ];
        let iter2 = SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
            sorted_archetypes: archetypes.clone(),
        });

        let result = iter1.clone().merge(iter2.clone());
        let reversed_result = iter1.merge(iter2.clone());

        assert_eq!(result, iter2);
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes().unwrap(), archetypes);
        assert_eq!(result.item_count(), 15 + 16);
    }

    #[test]
    fn merge_intersection_with_intersection() {
        let iter1 = SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
            sorted_archetypes: vec![
                create_archetype(2.into(), 15),
                create_archetype(4.into(), 17),
            ],
        });
        let iter2 = SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
            sorted_archetypes: vec![
                create_archetype(2.into(), 15),
                create_archetype(3.into(), 16),
            ],
        });

        let result = iter1.clone().merge(iter2.clone());
        let reversed_result = iter1.merge(iter2.clone());

        let archetypes = vec![create_archetype(2.into(), 15)];
        assert_eq!(
            result,
            SystemParamIterInfo::ComponentIntersectionEntities(EntityIterInfo {
                sorted_archetypes: archetypes.clone(),
            })
        );
        assert_eq!(result, reversed_result);
        assert_eq!(result.sorted_archetypes().unwrap(), archetypes);
        assert_eq!(result.item_count(), 15);
    }

    fn create_archetype(idx: ArchetypeIdx, entity_count: usize) -> ArchetypeInfo {
        ArchetypeInfo { idx, entity_count }
    }
}
