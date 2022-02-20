use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, GlobalAccess, SystemProperties};
use crate::system_params::globals::internal::{GlobalGuard, GlobalGuardBorrow};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::system_params::optional_globals::internal::GlobalOptionStream;
use crate::{Glob, Global, SystemData, SystemInfo, SystemParam};

#[allow(clippy::use_self)]
impl<'a, G> SystemParamWithLifetime<'a> for Option<Glob<'_, G>>
where
    G: Global,
{
    type Param = Option<Glob<'a, G>>;
    type Guard = GlobalGuard<'a, G>;
    type GuardBorrow = GlobalGuardBorrow<'a, G>;
    type Stream = GlobalOptionStream<'a, G>;
}

impl<G> SystemParam for Option<Glob<'_, G>>
where
    G: Global,
{
    type Tuple = (Self,);
    type InnerTuple = ();

    fn properties(core: &mut CoreStorage) -> SystemProperties {
        let idx = core.register_global::<G>();
        SystemProperties {
            component_types: vec![],
            globals: vec![GlobalAccess {
                access: Access::Read,
                idx,
            }],
            can_update: false,
            archetype_filter: ArchetypeFilter::None,
        }
    }

    fn lock<'a>(
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <Self as SystemParamWithLifetime<'a>>::Guard {
        GlobalGuard::new(data, info)
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
        GlobalOptionStream::new(guard)
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

impl<G> LockableSystemParam for Option<Glob<'_, G>>
where
    G: Global,
{
    type LockedType = Glob<'static, G>;
    type Mutability = Const;
}

mod internal {
    use crate::system_params::globals::internal::GlobalGuardBorrow;
    use crate::{Glob, Global};
    use std::ops::Range;

    pub struct GlobalOptionStream<'a, G> {
        global: Option<&'a G>,
        item_positions: Range<usize>,
    }

    impl<'a, G> GlobalOptionStream<'a, G>
    where
        G: Global,
    {
        pub(super) fn new(guard: &mut GlobalGuardBorrow<'a, G>) -> Self {
            Self {
                global: guard.global,
                item_positions: 0..guard.item_count,
            }
        }

        #[allow(clippy::option_option)]
        pub(super) fn next(&mut self) -> Option<Option<Glob<'_, G>>> {
            self.item_positions
                .next()
                .map(|_| self.global.map(|g| Glob { global: g }))
        }
    }
}

#[cfg(test)]
mod glob_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{Glob, Global, SystemInfo, SystemParam};
    use std::any::TypeId;

    #[derive(Debug, PartialEq)]
    struct TestGlobal(u32);

    impl Global for TestGlobal {}

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = Option::<Glob<'_, TestGlobal>>::properties(&mut core);
        assert_eq!(properties.component_types, vec![]);
        assert_eq!(properties.globals.len(), 1);
        assert_eq!(properties.globals[0].access, Access::Read);
        assert_eq!(properties.globals[0].idx, 0.into());
        assert!(!properties.can_update);
        assert_eq!(properties.archetype_filter, ArchetypeFilter::None);
    }

    #[test]
    fn use_system_param_when_existing() {
        let mut core = CoreStorage::default();
        core.create_entity_with_1_component(10_u32, None);
        core.create_entity_with_1_component(20_u32, None);
        core.create_entity_with_1_component(30_u32, None);
        core.replace_or_add_global(TestGlobal(10));
        let filtered_type_idx = core.components().type_idx(TypeId::of::<u32>()).unwrap();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 3,
        };
        let mut guard = Option::<Glob<'_, TestGlobal>>::lock(core.system_data(), info);
        let mut borrow = Option::<Glob<'_, TestGlobal>>::borrow_guard(&mut guard);
        let mut stream = Option::<Glob<'_, TestGlobal>>::stream(&mut borrow);
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert_eq!(item.unwrap().unwrap().0, 10);
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert_eq!(item.unwrap().unwrap().0, 10);
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert_eq!(item.unwrap().unwrap().0, 10);
        assert!(Option::<Glob<'_, _>>::stream_next(&mut stream).is_none());
    }

    #[test]
    fn use_system_param_when_missing() {
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
        let mut guard = Option::<Glob<'_, TestGlobal>>::lock(core.system_data(), info);
        let mut borrow = Option::<Glob<'_, TestGlobal>>::borrow_guard(&mut guard);
        let mut stream = Option::<Glob<'_, TestGlobal>>::stream(&mut borrow);
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert!(item.unwrap().is_none());
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert!(item.unwrap().is_none());
        let item = Option::<Glob<'_, _>>::stream_next(&mut stream);
        assert!(item.unwrap().is_none());
        assert!(Option::<Glob<'_, _>>::stream_next(&mut stream).is_none());
    }
}
