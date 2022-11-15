use crate::optional_singletons::internal::SingletonOptionStream;
use crate::singletons::internal::{SingletonGuard, SingletonGuardBorrow};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::{EntityMainComponent, Single, Singleton, SystemData, SystemInfo, SystemParam};

#[allow(clippy::use_self)]
impl<'a, C> SystemParamWithLifetime<'a> for Option<Single<'_, C>>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Param = Option<Single<'a, C>>;
    type Guard = SingletonGuard<'a, C>;
    type GuardBorrow = SingletonGuardBorrow<'a, C>;
    type Stream = SingletonOptionStream<'a, C>;
}

impl<C> SystemParam for Option<Single<'_, C>>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Filter = ();
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Read,
                type_idx,
            }],
            can_update: false,
        }
    }

    fn lock(
        data: SystemData<'_>,
        info: SystemInfo,
    ) -> <Self as SystemParamWithLifetime<'_>>::Guard {
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
    C: EntityMainComponent<Type = Singleton>,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::singletons::internal::SingletonGuardBorrow;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, EntityMainComponent, Single, Singleton, SystemData};
    use std::ops::Range;

    pub struct SingletonOptionStream<'a, C> {
        component: Option<(EntityIdx, &'a C)>,
        item_positions: Range<usize>,
        data: SystemData<'a>,
    }

    impl<'a, C> SingletonOptionStream<'a, C>
    where
        C: EntityMainComponent<Type = Singleton>,
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
