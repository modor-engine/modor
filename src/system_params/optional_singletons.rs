use crate::optional_singletons::internal::SingletonOptionStream;
use crate::singletons::internal::{SingletonGuard, SingletonGuardBorrow};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::{Single, SystemData, SystemInfo, SystemParam};
use std::any::Any;

#[allow(clippy::use_self)]
impl<'a, C> SystemParamWithLifetime<'a> for Option<Single<'_, C>>
where
    C: Any + Sync + Send,
{
    type Param = Option<Single<'a, C>>;
    type Guard = SingletonGuard<'a, C>;
    type GuardBorrow = SingletonGuardBorrow<'a, C>;
    type Stream = SingletonOptionStream<'a, C>;
}

impl<C> SystemParam for Option<Single<'_, C>>
where
    C: Any + Sync + Send,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx,
            }],
            can_update: false,
            archetype_filter: ArchetypeFilter::None,
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        SingletonGuard::new(data, info)
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
        SingletonOptionStream::new(guard)
    }

    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream.next()
    }
}

impl<C> LockableSystemParam for Option<Single<'_, C>>
where
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::singletons::internal::SingletonGuardBorrow;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, Single, SystemData};
    use std::any::Any;
    use std::ops::Range;

    pub struct SingletonOptionStream<'a, C> {
        component: Option<(EntityIdx, &'a C)>,
        item_positions: Range<usize>,
        data: SystemData<'a>,
    }

    impl<'a, C> SingletonOptionStream<'a, C>
    where
        C: Any + Sync + Send,
    {
        pub(super) fn new(guard: &'a mut SingletonGuardBorrow<'_, C>) -> Self {
            Self {
                component: (guard
                    .entity
                    .map(|(e, l)| (e, &guard.components[l.idx][l.pos]))),
                item_positions: 0..guard.item_count,
                data: guard.data,
            }
        }

        #[allow(clippy::option_option)]
        pub(super) fn next(&mut self) -> Option<Option<Single<'_, C>>> {
            self.item_positions.next().map(|_| {
                self.component.map(|(e, c)| Single {
                    component: c,
                    entity: Entity {
                        entity_idx: e,
                        data: self.data,
                    },
                })
            })
        }
    }
}

#[cfg(test)]
mod single_option_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{Single, SystemInfo, SystemParam};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = Option::<Single<'_, u32>>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.can_update);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn use_system_param_when_existing() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_i64, None);
        core.create_entity_with_1_component(20_i64, None);
        core.create_entity_with_1_component(30_i64, None);
        core.create_singleton(40_u32);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i64>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 3,
        };
        let mut guard = Option::<Single<'_, u32>>::lock(core.system_data(), info);
        let mut borrow = Option::<Single<'_, u32>>::borrow_guard(&mut guard);
        let mut stream = Option::<Single<'_, u32>>::stream(&mut borrow);
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(Some(&40)));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(Some(&40)));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(Some(&40)));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), None);
    }

    #[test]
    fn use_system_param_when_missing() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_i64, None);
        core.create_entity_with_1_component(20_i64, None);
        core.create_entity_with_1_component(30_i64, None);
        core.register_component_type::<u32>();
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i64>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 3,
        };
        let mut guard = Option::<Single<'_, u32>>::lock(core.system_data(), info);
        let mut borrow = Option::<Single<'_, u32>>::borrow_guard(&mut guard);
        let mut stream = Option::<Single<'_, u32>>::stream(&mut borrow);
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(None));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(None));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), Some(None));
        let item = Option::<Single<'_, u32>>::stream_next(&mut stream);
        assert_eq!(item.as_ref().map(Option::as_deref), None);
    }
}
