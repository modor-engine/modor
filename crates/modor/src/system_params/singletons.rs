use crate::singletons::internal::{SingletonGuard, SingletonGuardBorrow, SingletonStream};
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, ComponentTypeAccess, SystemProperties};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::{Entity, EntityMainComponent, Singleton, SystemData, SystemInfo, SystemParam};
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
/// # use modor::{Single, EntityMainComponent, Built, EntityBuilder, Singleton};
/// #
/// struct GameScore(u32);
///
/// impl EntityMainComponent for GameScore {
///     type Type = Singleton;
///     type Data = u32;
///
///     fn build(builder: EntityBuilder<'_, Self>, score: Self::Data) -> Built<'_> {
///         builder.with_self(Self(score))
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
            filtered_component_type_idxs: vec![],
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
    use crate::{Entity, EntityMainComponent, Single, Singleton, SystemData, SystemInfo};
    use std::any::{Any, TypeId};
    use std::ops::Range;
    use std::sync::RwLockReadGuard;

    pub struct SingletonGuard<'a, C> {
        components: RwLockReadGuard<'a, ComponentArchetypes<C>>,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a, C> SingletonGuard<'a, C>
    where
        C: Any,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                components: data.components.read_components::<C>(),
                data,
                info,
            }
        }

        pub(crate) fn borrow(&mut self) -> SingletonGuardBorrow<'_, C> {
            let type_idx = self
                .data
                .components
                .type_idx(TypeId::of::<C>())
                .expect("internal error: singleton type not registered");
            let singleton_location = self.data.components.singleton_locations(type_idx);
            SingletonGuardBorrow {
                components: &*self.components,
                item_count: self.info.item_count,
                entity: singleton_location
                    .map(|l| (self.data.archetypes.entity_idxs(l.idx)[l.pos], l)),
                data: self.data,
            }
        }
    }

    pub struct SingletonGuardBorrow<'a, C> {
        pub(crate) components: &'a ComponentArchetypes<C>,
        pub(crate) item_count: usize,
        pub(crate) entity: Option<(EntityIdx, EntityLocation)>,
        pub(crate) data: SystemData<'a>,
    }

    pub struct SingletonStream<'a, C> {
        component: Option<(EntityIdx, &'a C)>,
        item_positions: Range<usize>,
        data: SystemData<'a>,
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
                data: guard.data,
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
                        data: self.data,
                    },
                })
        }
    }
}

#[cfg(test)]
mod single_tests {
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{Entity, Single, Singleton, SystemInfo, SystemParam};
    use std::any::TypeId;

    create_entity_type!(SingletonEntity, Singleton);

    assert_impl_all!(Single<'_, SingletonEntity>: Sync, Send, Unpin);

    #[test]
    fn use_single() {
        let core = CoreStorage::default();
        let single = Single {
            component: &SingletonEntity(10),
            entity: Entity {
                entity_idx: 0.into(),
                data: core.system_data(),
            },
        };
        assert_eq!(&*single, &SingletonEntity(10));
        assert_eq!(single.entity().id(), 0);
    }

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = Single::<SingletonEntity>::properties(&mut core);
        assert_eq!(properties.component_types.len(), 1);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[0].type_idx, 0.into());
        assert!(!properties.can_update);
        assert_eq!(properties.filtered_component_type_idxs, []);
    }

    #[test]
    fn use_system_param_when_existing() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_i64, None);
        core.create_entity_with_1_component(20_i64, None);
        core.create_entity_with_1_component(30_i64, None);
        core.create_singleton(SingletonEntity(40));
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i64>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            item_count: 3,
        };
        let mut guard = Single::<SingletonEntity>::lock(core.system_data(), info);
        let mut borrow = Single::<SingletonEntity>::borrow_guard(&mut guard);
        let mut stream = Single::<SingletonEntity>::stream(&mut borrow);
        let item = Single::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&SingletonEntity(40)));
        let item = Single::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&SingletonEntity(40)));
        let item = Single::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&SingletonEntity(40)));
        assert_eq!(Single::stream_next(&mut stream).as_deref(), None);
    }

    #[test]
    fn use_system_param_when_missing() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_i64, None);
        core.create_entity_with_1_component(20_i64, None);
        core.create_entity_with_1_component(30_i64, None);
        core.register_component_type::<SingletonEntity>();
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i64>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            item_count: 3,
        };
        let mut guard = Single::<SingletonEntity>::lock(core.system_data(), info);
        let mut borrow = Single::<SingletonEntity>::borrow_guard(&mut guard);
        let mut stream = Single::<SingletonEntity>::stream(&mut borrow);
        assert_eq!(Single::stream_next(&mut stream).as_deref(), None);
    }
}
