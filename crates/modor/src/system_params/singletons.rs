use crate::singletons::internal::{SingletonGuard, SingletonGuardBorrow, SingletonStream};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::systems::context::SystemContext;
use crate::{Entity, EntityMainComponent, Singleton, SystemParam};
use std::ops::Deref;

/// A system parameter for immutably accessing the singleton of type `C`.
///
/// If the singleton does not exist, the system is not executed.<br>
/// If you want to execute the system even if the singleton does not exist, you can use instead a
/// system parameter of type `Option<Single<'_, C>>`.
///
/// # Examples
///
/// ```rust
/// # use modor::{singleton, Single, Built, EntityBuilder, Singleton};
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
/// fn print_score(score: Single<'_, GameScore>) {
///     println!("Score: {}", score.0);
/// }
/// ```
pub struct Single<'a, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    pub(crate) component: &'a C,
    pub(crate) entity: Entity<'a>,
}

impl<C> Single<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    /// Returns entity information.
    #[must_use]
    pub fn entity(&self) -> Entity<'_> {
        self.entity
    }
}

impl<C> Deref for Single<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<'a, C> SystemParamWithLifetime<'a> for Single<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Param = Single<'a, C>;
    type Guard = SingletonGuard<'a, C>;
    type GuardBorrow = SingletonGuardBorrow<'a, C>;
    type Stream = SingletonStream<'a, C>;
}

impl<C> SystemParam for Single<'_, C>
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
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        SingletonGuard::new(context)
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
        SingletonStream::new(guard)
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

impl<C> LockableSystemParam for Single<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type LockedType = C;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentArchetypes;
    use crate::storages::entities::EntityIdx;
    use crate::systems::context::SystemContext;
    use crate::{Entity, EntityMainComponent, Single, Singleton};
    use std::any::Any;
    use std::ops::Range;
    use std::sync::RwLockReadGuard;

    pub struct SingletonGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        context: SystemContext<'a>,
    }

    impl<'a, C> SingletonGuard<'a, C>
    where
        C: Any,
    {
        pub(crate) fn new(context: SystemContext<'a>) -> Self {
            Self {
                components: context.storages.components.read_components::<C>(),
                context,
            }
        }

        pub(crate) fn borrow(&mut self) -> SingletonGuardBorrow<'_, C> {
            let type_idx = self.context.component_type_idx::<C>();
            let singleton_location = self
                .context
                .storages
                .components
                .singleton_location(type_idx);
            SingletonGuardBorrow {
                components: &*self.components,
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

    pub struct SingletonGuardBorrow<'a, C> {
        pub(crate) components: &'a ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) entity: Option<(EntityIdx, EntityLocation)>,
        pub(crate) context: SystemContext<'a>,
    }

    pub struct SingletonStream<'a, C> {
        component: Option<(EntityIdx, &'a C)>,
        item_positions: Range<usize>,
        context: SystemContext<'a>,
    }

    impl<'a, C> SingletonStream<'a, C>
    where
        C: EntityMainComponent<Type = Singleton>,
    {
        pub(super) fn new(guard: &mut SingletonGuardBorrow<'a, C>) -> Self {
            Self {
                component: (guard
                    .entity
                    .map(|(e, l)| (e, &guard.components[l.idx][l.pos]))),
                item_positions: 0..guard.item_count,
                context: guard.context,
            }
        }

        pub(super) fn next(&mut self) -> Option<Single<'_, C>> {
            self.item_positions
                .next()
                .and(self.component)
                .map(|(e, c)| Single {
                    component: c,
                    entity: Entity {
                        entity_idx: e,
                        context: self.context,
                    },
                })
        }
    }
}
