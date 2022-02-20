use crate::storages::archetypes::ArchetypeFilter;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{Access, GlobalAccess, SystemProperties};
use crate::system_params::globals::internal::{GlobalGuard, GlobalGuardBorrow, GlobalStream};
use crate::system_params::internal::{Const, LockableSystemParam, SystemParamWithLifetime};
use crate::{Global, SystemData, SystemInfo, SystemParam};
use std::ops::Deref;

/// A system parameter for immutably accessing the global of type `G`.
///
/// If the global does not exist, the system is not executed.<br>
/// If you want to execute the system even if the global does not exist, you can use instead a
/// system parameter of type `Option<Glob<'_, G>>`.
///
/// # Examples
///
/// ```rust
/// # use modor::{Glob, Global};
/// #
/// struct GameScore(u32);
///
/// impl Global for GameScore {}
///
/// fn print_score(score: Glob<'_, GameScore>) {
///     println!("Score: {}", score.0);
/// }
/// ```
pub struct Glob<'a, G>
where
    G: Global,
{
    pub(crate) global: &'a G,
}

impl<G> Deref for Glob<'_, G>
where
    G: Global,
{
    type Target = G;

    fn deref(&self) -> &Self::Target {
        self.global
    }
}

impl<'a, G> SystemParamWithLifetime<'a> for Glob<'_, G>
where
    G: Global,
{
    type Param = Glob<'a, G>;
    type Guard = GlobalGuard<'a, G>;
    type GuardBorrow = GlobalGuardBorrow<'a, G>;
    type Stream = GlobalStream<'a, G>;
}

impl<G> SystemParam for Glob<'_, G>
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
        GlobalStream::new(guard)
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

impl<G> LockableSystemParam for Glob<'_, G>
where
    G: Global,
{
    type LockedType = Glob<'static, G>;
    type Mutability = Const;
}

pub(crate) mod internal {
    use crate::{Glob, Global, SystemData, SystemInfo};
    use std::any::Any;
    use std::ops::Range;
    use std::sync::RwLockReadGuard;

    pub struct GlobalGuard<'a, G> {
        global: Option<RwLockReadGuard<'a, G>>,
        info: SystemInfo<'a>,
    }

    impl<'a, G> GlobalGuard<'a, G>
    where
        G: Any,
    {
        pub(crate) fn new(data: SystemData<'a>, info: SystemInfo<'a>) -> Self {
            Self {
                global: data.globals.read::<G>(),
                info,
            }
        }

        pub(crate) fn borrow(&mut self) -> GlobalGuardBorrow<'_, G> {
            GlobalGuardBorrow {
                global: self.global.as_deref(),
                item_count: self.info.item_count,
            }
        }
    }

    pub struct GlobalGuardBorrow<'a, G> {
        pub(crate) global: Option<&'a G>,
        pub(crate) item_count: usize,
    }

    pub struct GlobalStream<'a, G> {
        global: Option<&'a G>,
        item_positions: Range<usize>,
    }

    impl<'a, G> GlobalStream<'a, G>
    where
        G: Global,
    {
        pub(super) fn new(guard: &mut GlobalGuardBorrow<'a, G>) -> Self {
            Self {
                global: guard.global,
                item_positions: 0..guard.item_count,
            }
        }

        pub(super) fn next(&mut self) -> Option<Glob<'_, G>> {
            self.item_positions
                .next()
                .and(self.global)
                .map(|g| Glob { global: g })
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
    fn use_glob() {
        let global = TestGlobal(10);
        let glob = Glob { global: &global };
        assert_eq!(&glob.0, &10);
    }

    #[test]
    fn retrieve_system_param_properties() {
        let mut core = CoreStorage::default();
        let properties = Glob::<TestGlobal>::properties(&mut core);
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
        let mut guard = Glob::<TestGlobal>::lock(core.system_data(), info);
        let mut borrow = Glob::<TestGlobal>::borrow_guard(&mut guard);
        let mut stream = Glob::<TestGlobal>::stream(&mut borrow);
        let item = Glob::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        let item = Glob::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        let item = Glob::stream_next(&mut stream);
        assert_eq!(item.as_deref(), Some(&TestGlobal(10)));
        assert_eq!(Glob::stream_next(&mut stream).as_deref(), None);
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
        let mut guard = Glob::<TestGlobal>::lock(core.system_data(), info);
        let mut borrow = Glob::<TestGlobal>::borrow_guard(&mut guard);
        let mut stream = Glob::<TestGlobal>::stream(&mut borrow);
        assert_eq!(Glob::stream_next(&mut stream).as_deref(), None);
    }
}
