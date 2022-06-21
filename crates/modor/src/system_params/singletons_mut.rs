use crate::singletons_mut::internal::{
    SingletonMutGuard, SingletonMutGuardBorrow, SingletonMutStream,
};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::{Entity, EntityMainComponent, Singleton, SystemData, SystemInfo, SystemParam};
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
/// # use modor::{singleton, Built, EntityBuilder, SingleMut, Singleton};
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
    C: EntityMainComponent<Type = Singleton>,
{
    pub(crate) component: &'a mut C,
    pub(crate) entity: Entity<'a>,
}

impl<C> SingleMut<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    /// Returns entity information.
    #[must_use]
    pub fn entity(&self) -> Entity<'_> {
        self.entity
    }
}

impl<C> Deref for SingleMut<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<C> DerefMut for SingleMut<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

impl<'a, C> SystemParamWithLifetime<'a> for SingleMut<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Param = SingleMut<'a, C>;
    type Guard = SingletonMutGuard<'a, C>;
    type GuardBorrow = SingletonMutGuardBorrow<'a, C>;
    type Stream = SingletonMutStream<'a, C>;
}

impl<C> SystemParam for SingleMut<'_, C>
where
    C: EntityMainComponent<Type = Singleton>,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let type_idx = core.register_component_type::<C>();
        SystemProperties {
            component_types: vec![ComponentTypeAccess {
                access: Access::Write,
                type_idx,
            }],
            can_update: false,
            filtered_component_type_idxs: vec![],
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        SingletonMutGuard::new(data, info)
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
    C: EntityMainComponent<Type = Singleton>,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentArchetypes;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, EntityMainComponent, SingleMut, Singleton, SystemData, SystemInfo};
    use std::any::{Any, TypeId};
    use std::ops::Range;
    use std::sync::RwLockWriteGuard;

    pub struct SingletonMutGuard<'a, C> {
        components: RwLockWriteGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> SingletonMutGuard<'a, C>
    where
        C: Any,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                components: data.components.write_components::<C>(),
                data,
                info,
            }
        }

        pub(crate) fn borrow(&mut self) -> SingletonMutGuardBorrow<'_, C> {
            let type_idx = self
                .data
                .components
                .type_idx(TypeId::of::<C>())
                .expect("internal error: singleton type not registered");
            let singleton_location = self.data.components.singleton_location(type_idx);
            SingletonMutGuardBorrow {
                components: &mut *self.components,
                item_count: self.info.item_count,
                entity: singleton_location
                    .map(|l| (self.data.archetypes.entity_idxs(l.idx)[l.pos], l)),
                data: self.data,
            }
        }
    }

    pub struct SingletonMutGuardBorrow<'a, C> {
        pub(crate) components: &'a mut ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) entity: Option<(EntityIdx, EntityLocation)>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct SingletonMutStream<'a, C> {
        component: Option<(EntityIdx, &'a mut C)>,
        item_positions: Range<usize>,
        data: SystemData<'a>,
    }

    impl<'a, C> SingletonMutStream<'a, C>
    where
        C: EntityMainComponent<Type = Singleton>,
    {
        pub(super) fn new(guard: &'a mut SingletonMutGuardBorrow<'_, C>) -> Self {
            Self {
                component: (guard
                    .entity
                    .map(|(e, l)| (e, &mut guard.components[l.idx][l.pos]))),
                item_positions: 0..guard.item_count,
                data: guard.data,
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
                        data: self.data,
                    },
                })
        }
    }
}
