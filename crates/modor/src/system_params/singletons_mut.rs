use crate::singletons_mut::internal::{
    SingletonMutGuard, SingletonMutGuardBorrow, SingletonMutStream,
};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::systems::context::SystemContext;
use crate::{Entity, EntityMainComponent, SystemParam, True};
use std::ops::{Deref, DerefMut};

/// A system parameter for mutably accessing the singleton of type `C`.
///
/// If the singleton does not exist, the system is not executed.<br>
/// If you want to execute the system even if the singleton does not exist, you can use instead a
/// system parameter of type `Option<SingleMut<'_, C>>`.
///
/// # Examples
///
/// ```rust
/// # use modor::{singleton, Built, EntityBuilder, SingleMut, True};
/// #
/// struct GameScore(u32);
///
/// #[singleton]
/// impl GameScore {
///     fn build(score: u32) -> impl Built<Self> {
///         EntityBuilder::new(Self(score))
///     }
/// }
///
/// fn increment_score(mut score: SingleMut<'_, GameScore>) {
///     score.0 += 1;
/// }
/// ```
pub struct SingleMut<'a, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    pub(crate) component: &'a mut C,
    pub(crate) entity: Entity<'a>,
}

impl<C> SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    /// Returns entity information.
    #[must_use]
    pub fn entity(&self) -> Entity<'_> {
        self.entity
    }
}

impl<C> Deref for SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<C> DerefMut for SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

impl<'a, C> SystemParamWithLifetime<'a> for SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    type Param = SingleMut<'a, C>;
    type Guard = SingletonMutGuard<'a, C>;
    type GuardBorrow = SingletonMutGuardBorrow<'a, C>;
    type Stream = SingletonMutStream<'a, C>;
}

impl<C> SystemParam for SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
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
        SingletonMutStream::new(guard)
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

impl<C> LockableSystemParam for SingleMut<'_, C>
where
    C: EntityMainComponent<IsSingleton = True>,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentArchetypes;
    use crate::storages::entities::EntityIdx;
    use crate::systems::context::SystemContext;
    use crate::{Entity, EntityMainComponent, SingleMut, True};
    use std::ops::Range;
    use std::sync::RwLockWriteGuard;

    pub struct SingletonMutGuard<'a, C> {
        components: RwLockWriteGuard<'a, ComponentArchetypes<C>>,
        context: SystemContext<'a>,
    }

    impl<'a, C> SingletonMutGuard<'a, C>
    where
        C: EntityMainComponent<IsSingleton = True>,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                components: context.storages.components.write_components::<C>(),
                context,
            }
        }

        pub(crate) fn borrow(&mut self) -> SingletonMutGuardBorrow<'_, C> {
            let type_idx = self.context.component_type_idx::<C>();
            let singleton_location = self
                .context
                .storages
                .components
                .singleton_location(type_idx);
            SingletonMutGuardBorrow {
                components: &mut *self.components,
                item_count: self.context.item_count,
                entity: singleton_location.map(|l| {
                    (
                        self.context.storages.archetypes.entity_idxs(l.idx)[l.pos],
                        l,
                    )
                }),
                context: self.context,
            }
        }
    }

    pub struct SingletonMutGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) entity: Option<(EntityIdx, EntityLocation)>,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct SingletonMutStream<'a, C> {
        component: Option<(EntityIdx, &'a mut C)>,
        item_positions: Range<usize>,
        context: SystemContext<'a>,
    }

    impl<'a, C> SingletonMutStream<'a, C>
    where
        C: EntityMainComponent<IsSingleton = True>,
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

        pub(super) fn next(&mut self) -> Option<SingleMut<'_, C>> {
            self.item_positions
                .next()
                .and(self.component.as_mut())
                .map(|(e, c)| SingleMut {
                    component: *c,
                    entity: Entity {
                        entity_idx: *e,
                        context: self.context,
                    },
                })
        }
    }
}
