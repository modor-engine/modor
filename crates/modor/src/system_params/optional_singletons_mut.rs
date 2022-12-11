use crate::optional_singletons_mut::internal::SingletonOptionMutStream;
use crate::singletons_mut::internal::{SingletonMutGuard, SingletonMutGuardBorrow};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::systems::context::SystemContext;
use crate::{EntityMainComponent, SingleMut, Singleton, SystemParam};

#[allow(clippy::use_self)]
impl<'a, C> SystemParamWithLifetime<'a> for Option<SingleMut<'_, C>>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Param = Option<SingleMut<'a, C>>;
    type Guard = SingletonMutGuard<'a, C>;
    type GuardBorrow = SingletonMutGuardBorrow<'a, C>;
    type Stream = SingletonOptionMutStream<'a, C>;
}

impl<C> SystemParam for Option<SingleMut<'_, C>>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Filter = ();
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Write,
                type_idx,
            }],
            can_update: false,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        SingletonMutGuard::new(context)
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
        SingletonOptionMutStream::new(guard)
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

impl<C> LockableSystemParam for Option<SingleMut<'_, C>>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::singletons_mut::internal::SingletonMutGuardBorrow;
    use crate::storages::entities::EntityIdx;
    use crate::systems::context::SystemContext;
    use crate::{Entity, EntityMainComponent, SingleMut, Singleton};
    use std::ops::Range;

    pub struct SingletonOptionMutStream<'a, C> {
        component: Option<(EntityIdx, &'a mut C)>,
        item_positions: Range<usize>,
        context: SystemContext<'a>,
    }

    impl<'a, C> SingletonOptionMutStream<'a, C>
    where
        C: EntityMainComponent<Type = Singleton>,
    {
        pub(super) fn new(guard: &'a mut SingletonMutGuardBorrow<'_, C>) -> Self {
            Self {
                component: if let Some((e, l)) = guard.entity {
                    let type_idx = guard.context.component_type_idx::<C>();
                    guard.context.add_mutated_component(type_idx, l.idx);
                    Some((e, &mut guard.components[l.idx][l.pos]))
                } else {
                    None
                },
                item_positions: 0..guard.item_count,
                context: guard.context,
            }
        }

        #[allow(clippy::option_option)]
        pub(super) fn next(&mut self) -> Option<Option<SingleMut<'_, C>>> {
            self.item_positions.next().map(|_| {
                self.component.as_mut().map(|(e, c)| SingleMut {
                    component: *c,
                    entity: Entity {
                        entity_idx: *e,
                        context: self.context,
                    },
                })
            })
        }
    }
}
