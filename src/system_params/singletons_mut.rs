use crate::singletons_mut::internal::{
    SingletonMutGuard, SingletonMutGuardBorrow, SingletonMutStream,
};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::{Entity, SystemData, SystemInfo, SystemParam};
use std::any::Any;
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
/// # use modor::{Built, EntityBuilder, EntityMainComponent, SingleMut};
/// #
/// struct GameScore(u32);
///
/// impl EntityMainComponent for GameScore {
///     type Type = ();
///     type Data = u32;
///
///     fn build(builder: EntityBuilder<'_, Self>, score: Self::Data) -> Built<'_> {
///         builder.with_self(Self(score))
///     }
/// }
///
/// fn increment_score(mut score: SingleMut<'_, GameScore>) {
///     score.0 += 1;
/// }
/// ```
pub struct SingleMut<'a, C>
where
    C: Any + Sync + Send,
{
    pub(crate) component: &'a mut C,
    pub(crate) entity: Entity<'a>,
}

impl<C> SingleMut<'_, C>
where
    C: Any + Sync + Send,
{
    /// Returns entity information.
    pub fn entity(&self) -> Entity<'_> {
        self.entity
    }
}

impl<C> Deref for SingleMut<'_, C>
where
    C: Any + Sync + Send,
{
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.component
    }
}

impl<C> DerefMut for SingleMut<'_, C>
where
    C: Any + Sync + Send,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.component
    }
}

impl<'a, C> SystemParamWithLifetime<'a> for SingleMut<'_, C>
where
    C: Any + Sync + Send,
{
    type Param = SingleMut<'a, C>;
    type Guard = SingletonMutGuard<'a, C>;
    type GuardBorrow = SingletonMutGuardBorrow<'a, C>;
    type Stream = SingletonMutStream<'a, C>;
}

impl<C> SystemParam for SingleMut<'_, C>
where
    C: Any + Sync + Send,
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
            archetype_filter: ArchetypeFilter::None,
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
    C: Any + Sync + Send,
{
    type LockedType = C;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentArchetypes;
    use crate::storages::entities::EntityIdx;
    use crate::{Entity, SingleMut, SystemData, SystemInfo};
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
            let singleton_location = self.data.components.singleton_locations(type_idx);
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
        C: Any + Sync + Send,
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

#[cfg(test)]
mod single_mut_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{Entity, SingleMut, SystemInfo, SystemParam};
    use std::any::TypeId;

    assert_impl_all!(SingleMut<'_, u32>: Sync, Send, Unpin);

    #[test]
    fn use_single() {
        let core = CoreStorage::default();
        let mut single = SingleMut {
            component: &mut 10_u32,
            entity: Entity {
                entity_idx: 0.into(),
                data: core.system_data(),
            },
        };
        assert_eq!(&*single, &10_u32);
        assert_eq!(&mut *single, &mut 10_u32);
        assert_eq!(single.entity().id(), 0);
    }

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = SingleMut::<u32>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Write);
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
        let mut guard = SingleMut::<u32>::lock(core.system_data(), info);
        let mut borrow = SingleMut::<u32>::borrow_guard(&mut guard);
        let mut stream = SingleMut::<u32>::stream(&mut borrow);
        let item = SingleMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&40));
        let item = SingleMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&40));
        let item = SingleMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&40));
        assert_eq!(SingleMut::stream_next(&mut stream).as_deref(), None);
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
        let mut guard = SingleMut::<u32>::lock(core.system_data(), info);
        let mut borrow = SingleMut::<u32>::borrow_guard(&mut guard);
        let mut stream = SingleMut::<u32>::stream(&mut borrow);
        assert_eq!(SingleMut::stream_next(&mut stream).as_deref(), None);
    }
}