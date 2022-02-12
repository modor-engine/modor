use crate::storages::actions::ActionStorage;
use crate::storages::archetypes::{
    ArchetypeFilter, ArchetypeIdx, ArchetypeStorage, FilteredArchetypeIdxIter,
};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::core::CoreStorage;
use crate::storages::entities::EntityStorage;
use crate::storages::systems::SystemProperties;
use crate::storages::updates::UpdateStorage;
use crate::system_params::internal::SystemParamWithLifetime;
use crate::systems::internal::{SealedSystem, SystemWrapper};
use crate::SystemParam;
use std::sync::Mutex;

/// Creates a valid instance of [`SystemBuilder`](crate::SystemBuilder).
///
/// The system passed as parameter must be a function or a static closure with no captured
/// variables, and must implement the [`System`](crate::System) trait.
///
/// # System behaviour
///
/// There are two types of system:
/// - iterative system: at least one of the argument types corresponds to an entity part
/// - non-iterative system: none of the argument types correspond to an entity part
///
/// The types that can represent an entity part are:
/// - `&C` where `C` is a component type
/// - `&mut C` where `C` is a component type
/// - `Option<&C>` where `C` is a component type
/// - `Option<&mut C>` where `C` is a component type
/// - [`Entity`](crate::Entity)
/// - a tuple containing at least one of the previous types
///
/// An iterative system is run for each entity containing components of type `C` when
/// `&C` or `&mut C` is the type of an argument. If there is no argument of type `&C` or `&mut C`,
/// then the system iterates on all entities.
///
/// A non-iterative system is only run once per application update.
///
/// # Static checks
///
/// Compile time checks are applied by this macro to ensure the system will not panic at runtime.
/// If the system is invalid, the macro returns a compile time error.
///
/// The [`SystemWithParams`](crate::SystemWithParams) trait is implemented for all systems.
///
/// The [`SystemWithParamMutabilityIssue`](crate::SystemWithParamMutabilityIssue) trait
/// is implemented in case the system is invalid. If this trait is implemented for the system,
/// it creates a compile time error due to a conflict with the implemented
/// [`SystemWithParams`](crate::SystemWithParams) trait.
///
/// # Limitation on the number of parameters
///
/// A system supports up to 10 parameters.<br>
/// If more parameters are needed, tuples can be used to group parameters and count as one.
///
/// # Examples
///
/// Valid systems:
/// ```rust
/// # use modor::{system, Entity, World, Query};
/// #
/// system!(iterative_system);
/// system!(other_iterative_system);
/// system!(non_iterative_system);
/// system!(iterative_system_again);
///
/// fn iterative_system(id: &u32, message: Option<&mut String>) {
///     // run for each entity with at least a component of type `u32`
///     // `String` is not used to filter entities as it is optional
///     if let Some(message) = message {
///         *message = format!("id: {}", id);
///     }
/// }
///
/// fn other_iterative_system(entity: Entity<'_>) {
///     // run for all entities
///     println!("entity detected with ID {}", entity.id());
/// }
///
/// fn non_iterative_system(mut world: World<'_>, query: Query<'_, Entity<'_>>) {
///     // run only once per application update
///     query.iter().for_each(|entity| world.delete_entity(entity.id()));
/// }
///
/// fn iterative_system_again(entity: Entity<'_>, mut world: World<'_>) {
///     // run for all entities because one of the parameters is of type `Entity`
///     // equivalent to the system `non_iterative_system`
///     world.delete_entity(entity.id());
/// }
/// ```
///
/// Invalid systems:
/// ```compile_fail
/// use modor::{system, Entity, World, Query};
///
/// system!(invalid_system);
///
/// fn invalid_system(name: &String, name_mut: &mut String) {
///     // invalid as `String` cannot be borrowed both mutably and immutably
///     *name_mut = format!("[[[ {} ]]]", name);
/// }
/// ```
#[macro_export]
macro_rules! system {
    ($system:expr) => {{
        use ::modor::{SystemWithParamMutabilityIssue, SystemWithParams};

        #[allow(clippy::semicolon_if_nothing_returned)]
        ::modor::SystemBuilder {
            properties_fn: ::modor::System::properties_fn(&$system),
            wrapper: |data: ::modor::SystemData<'_>, info: ::modor::SystemInfo<'_>| {
                let checker = ::modor::SystemParamMutabilityChecker::new($system);
                let mut system = checker.check_param_mutability().into_inner();
                let mut guard = ::modor::System::lock(&system, data, info);
                let mut guard_borrow = ::modor::System::borrow_guard(&system, &mut guard);
                let mut stream = ::modor::System::stream(&system, &mut guard_borrow);
                while let Some(item) = ::modor::System::stream_next(&system, &mut stream) {
                    ::modor::System::apply(&mut system, item);
                }
            },
        }
    }};
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemInfo<'a> {
    pub(crate) filtered_component_type_idxs: &'a [ComponentTypeIdx],
    pub(crate) archetype_filter: &'a ArchetypeFilter,
    pub(crate) item_count: usize,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct SystemData<'a> {
    pub(crate) entities: &'a EntityStorage,
    pub(crate) components: &'a ComponentStorage,
    pub(crate) archetypes: &'a ArchetypeStorage,
    pub(crate) actions: &'a ActionStorage,
    pub(crate) updates: &'a Mutex<UpdateStorage>,
}

impl SystemData<'_> {
    pub(crate) fn filter_archetype_idx_iter<'a>(
        &'a self,
        filtered_component_type_idxs: &'a [ComponentTypeIdx],
        archetype_filter: &'a ArchetypeFilter,
    ) -> FilteredArchetypeIdxIter<'a> {
        const EMPTY_ARCHETYPE_IDX_SLICE: &[ArchetypeIdx] = &[];
        let pre_filtered_archetype_idxs =
            if let Some(&type_idx) = filtered_component_type_idxs.first() {
                self.components.sorted_archetype_idxs(type_idx)
            } else {
                match &archetype_filter {
                    ArchetypeFilter::None => EMPTY_ARCHETYPE_IDX_SLICE,
                    ArchetypeFilter::All | ArchetypeFilter::Union(_) => {
                        self.archetypes.all_sorted_idxs()
                    }
                    ArchetypeFilter::Intersection(type_idxs) => {
                        self.components.sorted_archetype_idxs(*type_idxs.first())
                    }
                }
            };
        self.archetypes.filter_idxs(
            pre_filtered_archetype_idxs.iter(),
            filtered_component_type_idxs,
            archetype_filter,
        )
    }

    pub(crate) fn item_count(
        &self,
        filtered_component_type_idxs: &[ComponentTypeIdx],
        archetype_filter: &ArchetypeFilter,
    ) -> usize {
        if archetype_filter == &ArchetypeFilter::None {
            1
        } else {
            self.filter_archetype_idx_iter(filtered_component_type_idxs, archetype_filter)
                .map(|a| self.archetypes.entity_idxs(a).len())
                .sum()
        }
    }
}

/// A builder for defining a system.
///
/// The [`system!`](crate::system!) macro is used to construct a `SystemBuilder`.
pub struct SystemBuilder {
    #[doc(hidden)]
    pub properties_fn: fn(&mut CoreStorage) -> SystemProperties,
    #[doc(hidden)]
    pub wrapper: SystemWrapper,
}

/// A trait implemented for any system that can be passed to the [`system!`](crate::system!) macro.
pub trait System<P>: SealedSystem<P>
where
    P: SystemParam,
{
    #[doc(hidden)]
    fn properties_fn(&self) -> fn(&mut CoreStorage) -> SystemProperties {
        P::properties
    }

    #[doc(hidden)]
    fn lock<'a>(
        &self,
        data: SystemData<'a>,
        info: SystemInfo<'a>,
    ) -> <P as SystemParamWithLifetime<'a>>::Guard {
        P::lock(data, info)
    }

    #[doc(hidden)]
    fn borrow_guard<'a, 'b>(
        &self,
        guard: &'a mut <P as SystemParamWithLifetime<'b>>::Guard,
    ) -> <P as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        P::borrow_guard(guard)
    }

    #[doc(hidden)]
    fn stream<'a, 'b>(
        &self,
        guard: &'a mut <P as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <P as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        P::stream(guard)
    }

    #[doc(hidden)]
    #[inline]
    fn stream_next<'a, 'b>(
        &self,
        stream: &'a mut <P as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<P as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        P::stream_next(stream)
    }

    #[doc(hidden)]
    fn apply(&mut self, item: P);
}

macro_rules! impl_system {
    ($(($params:ident, $indexes:tt)),*) => {
        impl<$($params,)* S> SealedSystem<($($params,)*)> for S
        where
            S: FnMut($($params),*),
            $($params: SystemParam,)*
        {
        }

        impl<$($params,)* S> System<($($params,)*)> for S
        where
            S: FnMut($($params),*),
            $($params: SystemParam,)*
        {
            #[allow(unused_variables)]
            #[inline]
            fn apply(&mut self, item: ($($params,)*)) {
                self($(item.$indexes),*);
            }
        }
    };
}

impl_system!();
run_for_tuples_with_idxs!(impl_system);

pub(crate) mod internal {
    use crate::{SystemData, SystemInfo};

    pub trait SealedSystem<P> {}

    pub(crate) type SystemWrapper = fn(SystemData<'_>, SystemInfo<'_>);
}

#[cfg(test)]
mod system_data_tests {
    use crate::storages::archetypes::{ArchetypeFilter, ArchetypeStorage};
    use crate::storages::core::CoreStorage;

    #[test]
    fn retrieve_filter_archetype_idx_iter_when_some_filtered_types() {
        let mut core = CoreStorage::default();
        let (type1_idx, archetype1_idx) = core.add_component_type::<u32>(0.into());
        let (type2_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (_, location) = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type1_idx, location);
        core.add_component(20_i64, type2_idx, location);
        let data = core.system_data();
        let filtered_type_idxs = &[type2_idx];

        let mut iter = data.filter_archetype_idx_iter(filtered_type_idxs, &ArchetypeFilter::All);

        assert_eq!(iter.next(), Some(archetype2_idx));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_filter_archetype_idx_iter_when_no_filtered_type_and_none_archetype_filter() {
        let mut core = CoreStorage::default();
        let (type1_idx, archetype1_idx) = core.add_component_type::<u32>(0.into());
        let (type2_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (_, location) = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type1_idx, location);
        core.add_component(20_i64, type2_idx, location);
        let data = core.system_data();

        let mut iter = data.filter_archetype_idx_iter(&[], &ArchetypeFilter::None);

        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_filter_archetype_idx_iter_with_all_archetype_filter() {
        let mut core = CoreStorage::default();
        let (type1_idx, archetype1_idx) = core.add_component_type::<u32>(0.into());
        let (type2_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (_, location) = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type1_idx, location);
        core.add_component(20_i64, type2_idx, location);
        let data = core.system_data();

        let mut iter = data.filter_archetype_idx_iter(&[], &ArchetypeFilter::All);

        assert_eq!(iter.next(), Some(0.into()));
        assert_eq!(iter.next(), Some(archetype1_idx));
        assert_eq!(iter.next(), Some(archetype2_idx));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_filter_archetype_idx_iter_with_union_archetype_filter() {
        let mut core = CoreStorage::default();
        let (type1_idx, archetype1_idx) = core.add_component_type::<u32>(0.into());
        let (type2_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (_, location) = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type1_idx, location);
        core.add_component(20_i64, type2_idx, location);
        let data = core.system_data();
        let archetype_filter = ArchetypeFilter::Union(ne_vec![type1_idx]);

        let mut iter = data.filter_archetype_idx_iter(&[], &archetype_filter);

        assert_eq!(iter.next(), Some(archetype1_idx));
        assert_eq!(iter.next(), Some(archetype2_idx));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_filter_archetype_idx_iter_with_intersection_archetype_filter() {
        let mut core = CoreStorage::default();
        let (type1_idx, archetype1_idx) = core.add_component_type::<u32>(0.into());
        let (type2_idx, archetype2_idx) = core.add_component_type::<i64>(archetype1_idx);
        let (_, location) = core.create_entity(archetype2_idx);
        core.add_component(10_u32, type1_idx, location);
        core.add_component(20_i64, type2_idx, location);
        let data = core.system_data();
        let archetype_filter = ArchetypeFilter::Intersection(ne_vec![type1_idx]);

        let mut iter = data.filter_archetype_idx_iter(&[], &archetype_filter);

        assert_eq!(iter.next(), Some(archetype2_idx));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn retrieve_item_count_with_none_archetype_filter() {
        let core = CoreStorage::default();
        let data = core.system_data();

        let item_count = data.item_count(&[], &ArchetypeFilter::None);

        assert_eq!(item_count, 1);
    }

    #[test]
    fn retrieve_item_count_with_not_none_archetype_filter() {
        let mut core = CoreStorage::default();
        let (_, archetype_idx) = core.add_component_type::<u32>(0.into());
        core.create_entity(ArchetypeStorage::DEFAULT_IDX);
        core.create_entity(archetype_idx);
        core.create_entity(archetype_idx);
        let data = core.system_data();

        let item_count = data.item_count(&[], &ArchetypeFilter::All);

        assert_eq!(item_count, 3);
    }
}

#[cfg(test)]
mod system_builder_tests {
    use crate::SystemBuilder;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(SystemBuilder: Sync, Send, UnwindSafe, RefUnwindSafe, Unpin);
}

#[cfg(test)]
mod system_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::Access;
    use crate::{System, SystemInfo};
    use std::any::TypeId;

    #[test]
    fn retrieve_system_properties_fn() {
        let system = |_: &u32, _: &mut i64| ();

        let properties_fn = System::properties_fn(&system);

        let mut core = CoreStorage::default();
        let properties = properties_fn(&mut core);
        assert_eq!(properties.component_types.len(), 2);
        assert_eq!(properties.component_types[0].access, Access::Read);
        assert_eq!(properties.component_types[1].access, Access::Write);
        assert!(!properties.can_update);
    }

    #[test]
    fn use_system() {
        let mut core = CoreStorage::default();
        core.create_entity_with_2_components(10_u32, 100_i16);
        core.create_entity_with_2_components(20_u32, 200_i16);
        core.create_entity_with_1_component(30_u32);
        let filtered_type_idx = core.components().type_idx(TypeId::of::<i16>()).unwrap();
        let system = |_: &u32, _: &mut i16| ();
        let info = SystemInfo {
            filtered_component_type_idxs: &[filtered_type_idx],
            archetype_filter: &ArchetypeFilter::All,
            item_count: 2,
        };
        let mut guard = System::lock(&system, core.system_data(), info);
        let mut borrow = System::borrow_guard(&system, &mut guard);

        let mut stream = System::stream(&system, &mut borrow);
        let item = System::stream_next(&system, &mut stream);
        assert_eq!(item, Some((&10, &mut 100)));
        let item = System::stream_next(&system, &mut stream);
        assert_eq!(item, Some((&20, &mut 200)));
        assert_eq!(System::stream_next(&system, &mut stream), None);
    }

    macro_rules! test_apply {
        ([$($names:ident),*], [$($params:ident),*], [$($values:literal),*]) => {
            let mut collector = Vec::new();
            let mut system = |$($names: &$params),*| collector.push(($(*$names,)*));
            System::apply(&mut system, ($(&$values,)*));
            assert_eq!(collector, vec![($($values,)*)]);
        };
    }

    #[test]
    fn apply_params() {
        test_apply!([], [], []);
        test_apply!([a], [u8], [0]);
        test_apply!([a, b], [u8, u16], [0, 1]);
        test_apply!([a, b, c], [u8, u16, u32], [0, 1, 2]);
        test_apply!([a, b, c, d], [u8, u16, u32, u64], [0, 1, 2, 3]);
        test_apply!([a, b, c, d, e], [u8, u16, u32, u64, u128], [0, 1, 2, 3, 4]);
        test_apply!(
            [a, b, c, d, e, f],
            [u8, u16, u32, u64, u128, i8],
            [0, 1, 2, 3, 4, 5]
        );
        test_apply!(
            [a, b, c, d, e, f, g],
            [u8, u16, u32, u64, u128, i8, i16],
            [0, 1, 2, 3, 4, 5, 6]
        );
        test_apply!(
            [a, b, c, d, e, f, g, h],
            [u8, u16, u32, u64, u128, i8, i16, i32],
            [0, 1, 2, 3, 4, 5, 6, 7]
        );
        test_apply!(
            [a, b, c, d, e, f, g, h, i],
            [u8, u16, u32, u64, u128, i8, i16, i32, i64],
            [0, 1, 2, 3, 4, 5, 6, 7, 8]
        );
        test_apply!(
            [a, b, c, d, e, f, g, h, i, j],
            [u8, u16, u32, u64, u128, i8, i16, i32, i64, i128],
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
        );
    }
}
