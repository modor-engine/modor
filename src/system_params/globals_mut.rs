use crate::globals_mut::internal::{GlobalMutGuard, GlobalMutGuardBorrow, GlobalMutStream};
use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, GlobalAccess, SystemProperties};
use crate::system_params::internal::{LockableSystemParam, Mut, SystemParamWithLifetime};
use crate::{Glob, Global, SystemData, SystemInfo, SystemParam};
use std::ops::{Deref, DerefMut};

/// A system parameter for mutably accessing the global of type `G`.
///
/// If the global does not exist, the system is not executed.<br>
/// If you want to execute the system even if the global does not exist, you can use instead a
/// system parameter of type `Option<GlobMut<'_, G>>`.
///
/// # Examples
///
/// ```rust
/// # use modor::{Global, GlobMut};
/// #
/// struct GameScore(u32);
///
/// impl Global for GameScore {}
///
/// fn increment_score(mut score: GlobMut<'_, GameScore>) {
///     score.0 += 1;
/// }
/// ```
pub struct GlobMut<'a, G>
where
    G: Global,
{
    pub(crate) global: &'a mut G,
}

impl<G> Deref for GlobMut<'_, G>
where
    G: Global,
{
    type Target = G;

    fn deref(&self) -> &Self::Target {
        &*self.global
    }
}

impl<G> DerefMut for GlobMut<'_, G>
where
    G: Global,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.global
    }
}

impl<'a, G> SystemParamWithLifetime<'a> for GlobMut<'_, G>
where
    G: Global,
{
    type Param = GlobMut<'a, G>;
    type Guard = GlobalMutGuard<'a>;
    type GuardBorrow = GlobalMutGuardBorrow<'a>;
    type Stream = GlobalMutStream<'a, G>;
}

impl<G> SystemParam for GlobMut<'_, G>
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
                access: Access::Write,
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
        GlobalMutGuard::new(data, info)
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
        GlobalMutStream::new(guard)
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

impl<G> LockableSystemParam for GlobMut<'_, G>
where
    G: Global,
{
    type LockedType = Glob<'static, G>;
    type Mutability = Mut;
}

pub(crate) mod internal {
    use crate::{GlobMut, Global, SystemData, SystemInfo};
    use std::ops::Range;
    use std::sync::RwLockWriteGuard;

    pub struct GlobalMutGuard<'a> {
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    }

    impl<'a> GlobalMutGuard<'a> {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self { data, info }
        }

        pub(crate) fn borrow(&mut self) -> GlobalMutGuardBorrow<'_> {
            GlobalMutGuardBorrow {
                data: self.data,
                item_count: self.info.item_count,
            }
        }
    }

    pub struct GlobalMutGuardBorrow<'a> {
        pub(crate) data: SystemData<'a>,
        pub(crate) item_count: usize,
    }

    pub struct GlobalMutStream<'a, G> {
        global: Option<RwLockWriteGuard<'a, G>>,
        item_positions: Range<usize>,
    }

    impl<'a, G> GlobalMutStream<'a, G>
    where
        G: Global,
    {
        pub(super) fn new(guard: &mut GlobalMutGuardBorrow<'a>) -> Self {
            Self {
                global: guard.data.globals.write::<G>(),
                item_positions: 0..guard.item_count,
            }
        }

        pub(super) fn next(&mut self) -> Option<GlobMut<'_, G>> {
            self.item_positions
                .next()
                .and_then(|_| self.global.as_mut())
                .map(|g| GlobMut { global: &mut **g })
        }
    }
}

#[cfg(test)]
mod glob_mut_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{GlobMut, Global, SystemInfo, SystemParam};
    use std::any::TypeId;
    use std::panic::RefUnwindSafe;

    #[derive(Debug, PartialEq)]
    struct TestGlobal(u32);

    impl Global for TestGlobal {}

    assert_impl_all!(Glob<TestGlobal>: Sync, Send, Unpin, RefUnwindSafe);

    #[test]
    fn use_glob_mut() {
        let mut global = TestGlobal(10);
        let mut glob = GlobMut {
            global: &mut global,
        };
        assert_eq!(&glob.0, &10);
        assert_eq!(&mut glob.0, &mut 10);
    }

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = GlobMut::<TestGlobal>::properties(&mut core);
        assert_eq!(properties.component_types, vec![]);
        assert_eq!(properties.globals.len(), 1);
        assert_eq!(properties.globals[0].access, Access::Write);
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
        let mut guard = GlobMut::<TestGlobal>::lock(core.system_data(), info);
        let mut borrow = GlobMut::<TestGlobal>::borrow_guard(&mut guard);
        let mut stream = GlobMut::<TestGlobal>::stream(&mut borrow);
        let item = GlobMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        let item = GlobMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        let item = GlobMut::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        assert_eq!(GlobMut::stream_next(&mut stream).as_deref(), None);
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
        let mut guard = GlobMut::<TestGlobal>::lock(core.system_data(), info);
        let mut borrow = GlobMut::<TestGlobal>::borrow_guard(&mut guard);
        let mut stream = GlobMut::<TestGlobal>::stream(&mut borrow);
        assert_eq!(GlobMut::stream_next(&mut stream).as_deref(), None);
    }
}
