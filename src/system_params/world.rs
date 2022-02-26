use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::system_params::world::internal::{WorldGuard, WorldStream};
use crate::world::internal::WorldGuardBorrow;
use crate::{App, EntityBuilder, EntityMainComponent, Global, SystemData, SystemInfo, SystemParam};
use std::any::{Any, TypeId};

/// A system parameter for applying actions on entities.
///
/// # Examples
///
/// ```rust
/// # use modor::{Entity, World};
/// #
/// fn add_string_component(mut world: World<'_>, entity: Entity<'_>) {
///     let component = format!("entity_{}", entity.id());
///     world.add_component(entity.id(), component);
/// }
/// ```
pub struct World<'a> {
    data: SystemData<'a>,
}

impl<'a> World<'a> {
    /// Creates a new root entity of type `E` with building data `data`.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_root_entity<E>(&mut self, data: E::Data)
    where
        E: EntityMainComponent,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create root entity")
            .create_entity(
                None,
                Box::new(|c| {
                    E::build(EntityBuilder::<_, ()>::new(c, None), data);
                }),
            );
    }

    /// Creates a new entity of type `E` with building data `data` and parent entity with ID
    /// `parent_id`.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_child_entity<E>(&mut self, parent_id: usize, data: E::Data)
    where
        E: EntityMainComponent,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create child entity")
            .create_entity(
                Some(parent_id.into()),
                Box::new(move |c| {
                    E::build(EntityBuilder::<_, ()>::new(c, Some(parent_id.into())), data);
                }),
            );
    }

    /// Deletes an entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete_entity(&mut self, entity_id: usize) {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to delete entity")
            .delete_entity(entity_id.into());
    }

    /// Adds a component of type `C` to an entity.
    ///
    /// The component is actually added once all registered systems have been run.
    ///
    /// If the entity already has a component of type `C`, it is overwritten.<br>
    /// If `C` implements [`EntityMainComponent`](crate::EntityMainComponent),
    /// systems defined for `C` will now be run for the entity.
    pub fn add_component<C>(&mut self, entity_id: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to add component")
            .add_component(
                entity_id.into(),
                |c, a| c.add_component_type::<C>(a).1,
                Box::new(move |c, l| {
                    let type_idx = c
                        .components()
                        .type_idx(TypeId::of::<C>())
                        .expect("internal error: add component with not registered type");
                    c.add_component::<C>(component, type_idx, l, false);
                }),
            );
    }

    /// Deletes the component of type `C` from an entity.
    ///
    /// The component is actually deleted once all registered systems have been run.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.<br>
    /// If `C` implements [`EntityMainComponent`](crate::EntityMainComponent),
    /// systems defined for `C` will not be run anymore for the entity.
    pub fn delete_component<C>(&mut self, entity_id: usize)
    where
        C: Any + Sync + Send,
    {
        if let Some(type_idx) = self.data.components.type_idx(TypeId::of::<C>()) {
            self.data
                .updates
                .try_lock()
                .expect("internal error: cannot lock updates to delete component")
                .delete_component(entity_id.into(), type_idx);
        }
    }

    /// Creates a new global of type `G`.
    ///
    /// The global is actually created once all registered systems have been run.
    ///
    /// If a global of type `G` already exists, it is overwritten.
    pub fn create_global<G>(&mut self, global: G)
    where
        G: Global,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to create global")
            .create_global(Box::new(|c| App::create_global(c, global)));
    }

    /// Deletes the global of type `G`.
    ///
    /// The global is actually deleted once all registered systems have been run.
    ///
    /// If no global of type `G` exist, nothing is done.
    pub fn delete_global<G>(&mut self)
    where
        G: Global,
    {
        self.data
            .updates
            .try_lock()
            .expect("internal error: cannot lock updates to delete global")
            .delete_global(TypeId::of::<G>());
    }
}

impl<'a> SystemParamWithLifetime<'a> for World<'_> {
    type Param = World<'a>;
    type Guard = WorldGuard<'a>;
    type GuardBorrow = WorldGuardBorrow<'a>;
    type Stream = WorldStream<'a>;
}

impl SystemParam for World<'_> {
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            globals: vec![],
            can_update: true,
            archetype_filter: ArchetypeFilter::None,
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        WorldGuard::new(data, info)
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
        WorldStream::new(guard)
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        stream
            .item_positions
            .next()
            .map(move |_| World { data: stream.data })
    }
}

impl LockableSystemParam for World<'_> {
    type LockedType = World<'static>;
    type Mutability = Mut;
}

mod internal {
    use crate::{SystemData, SystemInfo};
    use std::ops::Range;

    pub struct WorldGuard<'a> {
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a> WorldGuard<'a> {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self { data, info }
        }

        pub(crate) fn borrow(&mut self) -> WorldGuardBorrow<'_> {
            WorldGuardBorrow {
                item_count: self.info.item_count,
                data: self.data,
            }
        }
    }

    pub struct WorldGuardBorrow<'a> {
        pub(crate) item_count: usize,
        pub(crate) data: SystemData<'a>,
    }

    pub struct WorldStream<'a> {
        pub(crate) data: SystemData<'a>,
        pub(crate) item_positions: Range<usize>,
    }

    impl<'a> WorldStream<'a> {
        pub(crate) fn new(guard: &'a WorldGuardBorrow<'_>) -> Self {
            Self {
                data: guard.data,
                item_positions: 0..guard.item_count,
            }
        }
    }
}

#[cfg(test)]
mod world_tests {
    use crate::storages::archetypes::{ArchetypeFilter, ArchetypeStorage};
    use crate::storages::core::CoreStorage;
    use crate::{
        Built, EntityBuilder, EntityMainComponent, Global, SystemInfo, SystemParam, World,
    };
    use std::any::TypeId;

    #[derive(Debug, PartialEq, Clone)]
    struct TestEntity(u32);

    impl EntityMainComponent for TestEntity {
        type Type = ();
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
            builder.with_self(Self(data))
        }
    }

    struct TestGlobal1(u32);

    impl Global for TestGlobal1 {}

    #[derive(Debug, PartialEq, Clone)]
    struct TestGlobal2(u32);

    impl Global for TestGlobal2 {}

    assert_impl_all!(World<'_>: Sync, Send, Unpin);

    #[test]
    fn use_world() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_u32, None);
        core.create_entity(ArchetypeStorage::DEFAULT_IDX, None);
        core.create_entity_with_1_component(20_i8, None);
        core.replace_or_add_global(TestGlobal1(60));
        let data = core.system_data();
        let mut world = World { data };
        world.delete_entity(0);
        world.add_component(1, 30_i8);
        world.delete_component::<i8>(2);
        world.create_root_entity::<TestEntity>(40);
        world.create_child_entity::<TestEntity>(1, 50);
        world.create_global(TestGlobal2(70));
        world.delete_global::<TestGlobal1>();
        core.update();
        assert_eq!(core.entities().location(0.into()), None);
        let components = core.components().read_components::<i8>().clone();
        assert_eq!(components, ti_vec![ti_vec![], ti_vec![], ti_vec![30_i8]]);
        let components = core.components().read_components::<TestEntity>().clone();
        let new_entities = ti_vec![TestEntity(50), TestEntity(40)];
        assert_eq!(
            components,
            ti_vec![ti_vec![], ti_vec![], ti_vec![], new_entities]
        );
        assert_eq!(core.entities().parent_idx(3.into()), Some(1.into()));
        let global = core.globals().read::<TestGlobal2>().unwrap().clone();
        assert_eq!(global, TestGlobal2(70));
        assert!(core.globals().read::<TestGlobal1>().is_none());
    }

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = World::properties(&mut core);
        assert_eq!(properties.component_types.len(), 0);
        assert_eq!(properties.globals, vec![]);
        assert!(properties.can_update);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn use_system_param() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_u32, None);
        core.create_entity_with_1_component(20_u32, None);
        core.create_entity_with_1_component(30_u32, None);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<u32>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 3,
        };
        let mut guard = World::lock(core.system_data(), info);
        let mut borrow = World::borrow_guard(&mut guard);
        let mut stream = World::stream(&mut borrow);
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_some());
        assert!(World::stream_next(&mut stream).is_none());
    }
}
