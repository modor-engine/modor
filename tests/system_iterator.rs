#![allow(clippy::print_stdout)]

use fxhash::FxHashMap;
use std::any::{Any, TypeId};
use std::iter;
use std::iter::{FlatMap, Map, Repeat, Take, Zip};
use std::marker::PhantomData;
use std::ops::Range;
use std::option::Option::Some;
use std::slice::{Iter, IterMut};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

macro_rules! nested_tuple {
    ($($param:ident),+) => {
        nested_tuple!(@internal (), ($($param),+))
    };
    (@internal ($($lefts:ident),*), ($right:ident $(,$rights:ident)+)) => {
        nested_tuple!(@internal ($($lefts,)* $right), ($($rights),+))
    };
    (@internal ($($lefts:ident),+), ($right:ident)) => {
        (nested_tuple!(@internal (), ($($lefts),+)), $right)
    };
    (@internal (), ($right:ident)) => {
        $right
    };
}

struct ArchetypeInfo {
    idx: usize,
    entity_count: usize,
}

struct SystemData<'a> {
    components: &'a [Box<dyn Any>],
    type_idxs: &'a FxHashMap<TypeId, usize>,
}

impl SystemData<'_> {
    #[allow(clippy::unused_self)]
    fn sorted_archetypes(&self, _component_type_idxs: &[TypeId]) -> Vec<ArchetypeInfo> {
        vec![
            ArchetypeInfo {
                idx: 0,
                entity_count: 3,
            },
            ArchetypeInfo {
                idx: 1,
                entity_count: 3,
            },
        ]
    }
}

struct SystemRunner<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
    T: SystemParam<'a, 'b>,
{
    system: &'a mut S,
    item_count: usize,
    guard: T::Guard,
    sorted_archetypes: Vec<ArchetypeInfo>,
}

impl<'a, 'b, S, T> SystemRunner<'a, 'b, S, T>
where
    S: System<'a, 'b, T>,
    T: SystemParam<'a, 'b>,
{
    fn new(system: &'a mut S, data: &'a SystemData<'_>) -> Self {
        let guard = T::lock(data);
        let sorted_archetype_idxs = data.sorted_archetypes(&T::mandatory_component_types());
        Self {
            system,
            item_count: T::item_count(&guard, &sorted_archetype_idxs),
            guard,
            sorted_archetypes: sorted_archetype_idxs,
        }
    }

    fn run(&'b mut self) {
        let data = SystemParamDataMut {
            guard: &mut self.guard,
            sorted_archetypes: &self.sorted_archetypes,
            item_count: self.item_count,
        };
        for item in T::iter_mut(data) {
            S::APPLY_FN(&mut self.system, item);
        }
    }
}

// TODO: add method to retrieve useful information to register the system
//     (create other trait if possible to simplify macro generation)
trait System<'a, 'b, T>
where
    T: SystemParam<'a, 'b>,
{
    const APPLY_FN: fn(&mut Self, T);
}

impl<'a, 'b, S, A, B> System<'a, 'b, (A, B)> for S
where
    'a: 'b,
    S: FnMut(A, B),
    A: SystemParam<'a, 'b>,
    B: SystemParam<'a, 'b>,
{
    const APPLY_FN: fn(&mut Self, (A, B)) = |s, t| s(t.0, t.1);
}

struct SystemParamData<'a, G> {
    guard: &'a G,
    sorted_archetypes: &'a [ArchetypeInfo],
    item_count: usize,
}

macro_rules! map_system_param_data {
    ($data:ident, $guard_field:tt) => {
        SystemParamData {
            guard: &$data.guard.$guard_field,
            sorted_archetypes: $data.sorted_archetypes,
            item_count: $data.item_count,
        }
    };
}

struct SystemParamDataMut<'a, G> {
    guard: &'a mut G,
    sorted_archetypes: &'a [ArchetypeInfo],
    item_count: usize,
}

macro_rules! map_system_param_data_mut {
    ($data:ident, $guard_field:tt) => {
        SystemParamDataMut {
            guard: &mut $data.guard.$guard_field,
            sorted_archetypes: $data.sorted_archetypes,
            item_count: $data.item_count,
        }
    };
}

trait SystemParam<'a, 'b>: 'b {
    type Guard: 'a;
    type Const: 'b;
    type Iter: Iterator<Item = Self::Const> + 'b + DoubleEndedIterator + ExactSizeIterator;
    type IterMut: Iterator<Item = Self> + 'b + DoubleEndedIterator + ExactSizeIterator;

    fn mandatory_component_types() -> Vec<TypeId>;

    fn lock(data: &'a SystemData<'_>) -> Self::Guard;

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize;

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter;

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut;
}

impl<'a, 'b, A, B> SystemParam<'a, 'b> for (A, B)
where
    'a: 'b,
    A: SystemParam<'a, 'b>,
    B: SystemParam<'a, 'b>,
{
    type Guard = (A::Guard, B::Guard);
    type Const = (A::Const, B::Const);
    #[allow(clippy::type_complexity)]
    type Iter = Map<Zip<A::Iter, B::Iter>, fn((A::Const, B::Const)) -> (A::Const, B::Const)>;
    #[allow(clippy::type_complexity)]
    type IterMut = Map<Zip<A::IterMut, B::IterMut>, fn((A, B)) -> (A, B)>;

    fn mandatory_component_types() -> Vec<TypeId> {
        let mut types = Vec::new();
        types.extend(A::mandatory_component_types());
        types.extend(B::mandatory_component_types());
        types
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        (A::lock(data), B::lock(data))
    }

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        0.max(A::item_count(&guard.0, archetypes))
            .max(B::item_count(&guard.1, archetypes))
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        A::iter(map_system_param_data!(data, 0))
            .zip(B::iter(map_system_param_data!(data, 1)))
            .map(|nested_tuple!(a, b)| (a, b))
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        A::iter_mut(map_system_param_data_mut!(data, 0))
            .zip(B::iter_mut(map_system_param_data_mut!(data, 1)))
            .map(|nested_tuple!(a, b)| (a, b))
    }
}

impl<'a, 'b, A, B, C, D> SystemParam<'a, 'b> for (A, B, C, D)
where
    'a: 'b,
    A: SystemParam<'a, 'b>,
    B: SystemParam<'a, 'b>,
    C: SystemParam<'a, 'b>,
    D: SystemParam<'a, 'b>,
{
    type Guard = (A::Guard, B::Guard, C::Guard, D::Guard);
    type Const = (A::Const, B::Const, C::Const, D::Const);
    #[allow(clippy::type_complexity)]
    type Iter = Map<
        Zip<Zip<Zip<A::Iter, B::Iter>, C::Iter>, D::Iter>,
        fn(
            (((A::Const, B::Const), C::Const), D::Const),
        ) -> (A::Const, B::Const, C::Const, D::Const),
    >;
    #[allow(clippy::type_complexity)]
    type IterMut = Map<
        Zip<Zip<Zip<A::IterMut, B::IterMut>, C::IterMut>, D::IterMut>,
        fn((((A, B), C), D)) -> (A, B, C, D),
    >;

    fn mandatory_component_types() -> Vec<TypeId> {
        let mut types = Vec::new();
        types.extend(A::mandatory_component_types());
        types.extend(B::mandatory_component_types());
        types
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        (A::lock(data), B::lock(data), C::lock(data), D::lock(data))
    }

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        0.max(A::item_count(&guard.0, archetypes))
            .max(B::item_count(&guard.1, archetypes))
            .max(C::item_count(&guard.2, archetypes))
            .max(D::item_count(&guard.3, archetypes))
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        A::iter(map_system_param_data!(data, 0))
            .zip(B::iter(map_system_param_data!(data, 1)))
            .zip(C::iter(map_system_param_data!(data, 2)))
            .zip(D::iter(map_system_param_data!(data, 3)))
            .map(|nested_tuple!(a, b, c, d)| (a, b, c, d))
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        A::iter_mut(map_system_param_data_mut!(data, 0))
            .zip(B::iter_mut(map_system_param_data_mut!(data, 1)))
            .zip(C::iter_mut(map_system_param_data_mut!(data, 2)))
            .zip(D::iter_mut(map_system_param_data_mut!(data, 3)))
            .map(|nested_tuple!(a, b, c, d)| (a, b, c, d))
    }
}

struct Archetype {
    idx: usize,
}

impl<'a, 'b> SystemParam<'a, 'b> for Archetype {
    type Guard = &'a SystemData<'a>;
    type Const = Self;
    type Iter = ArchetypeIter<'b>;
    type IterMut = ArchetypeIter<'b>;

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        data
    }

    fn item_count(_guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        archetypes.iter().map(|a| a.entity_count).sum()
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        ArchetypeIter {
            iter: data
                .sorted_archetypes
                .iter()
                .flat_map(|a| LimitedRepeat::new(a, a.entity_count).map(|a| Self { idx: a.idx })),
            len: data.item_count,
        }
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        ArchetypeIter {
            iter: data
                .sorted_archetypes
                .iter()
                .flat_map(|a| LimitedRepeat::new(a, a.entity_count).map(|a| Self { idx: a.idx })),
            len: data.item_count,
        }
    }
}

struct ArchetypeIter<'a> {
    #[allow(clippy::type_complexity)]
    iter: FlatMap<
        Iter<'a, ArchetypeInfo>,
        Map<LimitedRepeat<&'a ArchetypeInfo>, fn(&ArchetypeInfo) -> Archetype>,
        fn(&ArchetypeInfo) -> Map<LimitedRepeat<&ArchetypeInfo>, fn(&ArchetypeInfo) -> Archetype>,
    >,
    len: usize,
}

impl Iterator for ArchetypeIter<'_> {
    type Item = Archetype;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl DoubleEndedIterator for ArchetypeIter<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl ExactSizeIterator for ArchetypeIter<'_> {
    fn len(&self) -> usize {
        self.len
    }
}

struct LimitedRepeat<T>(Take<Repeat<T>>);

impl<T> LimitedRepeat<T>
where
    T: Clone,
{
    fn new(item: T, count: usize) -> Self {
        Self(iter::repeat(item).take(count))
    }
}

impl<T> Iterator for LimitedRepeat<T>
where
    T: Clone,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T> DoubleEndedIterator for LimitedRepeat<T>
where
    T: Clone,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a, 'b, C> SystemParam<'a, 'b> for &'b C
where
    C: Any,
{
    type Guard = RwLockReadGuard<'a, Vec<Vec<C>>>;
    type Const = Self;
    type Iter = ComponentIter<'b, C>;
    type IterMut = Self::Iter;

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        let type_idx = data.type_idxs[&TypeId::of::<C>()];
        data.components[type_idx]
            .downcast_ref::<RwLock<Vec<Vec<C>>>>()
            .expect("Wrong type index used")
            .try_read()
            .expect("Resource already locked")
    }

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        archetypes.iter().map(|a| guard[a.idx].len()).sum()
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        ComponentIter {
            components: ComponentArchetypeIter {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter(),
            }
            .flat_map(|c| c.iter()),
            len: data.item_count,
        }
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        ComponentIter {
            components: ComponentArchetypeIter {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter(),
            }
            .flat_map(|c| c.iter()),
            len: data.item_count,
        }
    }
}

struct ComponentArchetypeIter<'a, C> {
    sorted_archetypes: Iter<'a, ArchetypeInfo>,
    last_archetype_idx: Option<usize>,
    component_archetypes: Iter<'a, Vec<C>>,
}

impl<'a, C> Iterator for ComponentArchetypeIter<'a, C> {
    type Item = &'a Vec<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next()?.idx;
        let nth = archetype_idx - self.last_archetype_idx.map_or(0, |i| i + 1);
        self.last_archetype_idx = Some(archetype_idx);
        self.component_archetypes.nth(nth)
    }
}

impl<'a, C> DoubleEndedIterator for ComponentArchetypeIter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next_back()?.idx;
        let nth_back = self.sorted_archetypes.len() - archetype_idx;
        self.component_archetypes.nth_back(nth_back)
    }
}

struct ComponentIter<'a, C> {
    #[allow(clippy::type_complexity)]
    components: FlatMap<ComponentArchetypeIter<'a, C>, Iter<'a, C>, fn(&Vec<C>) -> Iter<'_, C>>,
    len: usize,
}

impl<'a, C> Iterator for ComponentIter<'a, C> {
    type Item = &'a C;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.components.next()
    }
}

impl<'a, C> DoubleEndedIterator for ComponentIter<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.components.next_back()
    }
}

impl<'a, C> ExactSizeIterator for ComponentIter<'a, C> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, 'b, C> SystemParam<'a, 'b> for &'b mut C
where
    C: Any,
{
    type Guard = RwLockWriteGuard<'a, Vec<Vec<C>>>;
    type Const = &'b C;
    type Iter = ComponentIter<'b, C>;
    type IterMut = ComponentIterMut<'b, C>;

    fn mandatory_component_types() -> Vec<TypeId> {
        vec![TypeId::of::<C>()]
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        let type_idx = data.type_idxs[&TypeId::of::<C>()];
        data.components[type_idx]
            .downcast_ref::<RwLock<Vec<Vec<C>>>>()
            .expect("Wrong type index used")
            .try_write()
            .expect("Resource already locked")
    }

    fn item_count(guard: &Self::Guard, archetypes: &[ArchetypeInfo]) -> usize {
        archetypes.iter().map(|a| guard[a.idx].len()).sum()
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        ComponentIter {
            components: ComponentArchetypeIter {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter(),
            }
            .flat_map(|c| c.iter()),
            len: data.item_count,
        }
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        ComponentIterMut {
            components: ComponentArchetypeIterMut {
                sorted_archetypes: data.sorted_archetypes.iter(),
                last_archetype_idx: None,
                component_archetypes: data.guard.iter_mut(),
            }
            .flat_map(|c| c.iter_mut()),
            len: data.item_count,
        }
    }
}

struct ComponentArchetypeIterMut<'a, C> {
    sorted_archetypes: Iter<'a, ArchetypeInfo>,
    last_archetype_idx: Option<usize>,
    component_archetypes: IterMut<'a, Vec<C>>,
}

impl<'a, C> Iterator for ComponentArchetypeIterMut<'a, C> {
    type Item = &'a mut Vec<C>;

    fn next(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next()?.idx;
        let nth = archetype_idx - self.last_archetype_idx.map_or(0, |i| i + 1);
        self.last_archetype_idx = Some(archetype_idx);
        self.component_archetypes.nth(nth)
    }
}

impl<'a, C> DoubleEndedIterator for ComponentArchetypeIterMut<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let archetype_idx = self.sorted_archetypes.next_back()?.idx;
        let nth_back = self.sorted_archetypes.len() - archetype_idx;
        self.component_archetypes.nth_back(nth_back)
    }
}

struct ComponentIterMut<'a, C> {
    #[allow(clippy::type_complexity)]
    components: FlatMap<
        ComponentArchetypeIterMut<'a, C>,
        IterMut<'a, C>,
        fn(&mut Vec<C>) -> IterMut<'_, C>,
    >,
    len: usize,
}

impl<'a, C> Iterator for ComponentIterMut<'a, C> {
    type Item = &'a mut C;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.components.next()
    }
}

impl<'a, C> DoubleEndedIterator for ComponentIterMut<'a, C> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.components.next_back()
    }
}

impl<'a, C> ExactSizeIterator for ComponentIterMut<'a, C> {
    fn len(&self) -> usize {
        self.len
    }
}

struct Query<'a, T>
where
    T: QueryParam,
{
    data: &'a SystemData<'a>,
    phantom: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: QueryParam,
{
    fn lock<'b>(&self) -> QueryGuard<'a, 'b, T::Const>
    where
        'a: 'b,
        T: SystemParam<'a, 'b>,
        T::Const: SystemParam<'a, 'b>,
    {
        let sorted_archetypes = self.data.sorted_archetypes(&T::mandatory_component_types());
        let guard = T::Const::lock(self.data);
        QueryGuard {
            item_count: T::Const::item_count(&guard, &sorted_archetypes),
            sorted_archetypes,
            guard,
        }
    }

    fn lock_mut<'b, 'c>(&'b mut self) -> QueryGuardMut<'b, 'c, T::WithLifetimes>
    where
        'b: 'c,
        T: SystemParamWithLifetimes<'b, 'c>,
    {
        let sorted_archetypes = self
            .data
            .sorted_archetypes(&T::WithLifetimes::mandatory_component_types());
        let guard = T::WithLifetimes::lock(self.data);
        QueryGuardMut {
            item_count: T::WithLifetimes::item_count(&guard, &sorted_archetypes),
            sorted_archetypes,
            guard,
        }
    }
}

impl<'a, 'b, T> SystemParam<'a, 'b> for Query<'b, T>
where
    'a: 'b,
    T: QueryParam + SystemParam<'a, 'b>,
    T::Const: QueryParam,
{
    type Guard = &'a SystemData<'a>;
    type Const = Query<'b, T::Const>;
    type Iter = QueryIter<'b, T::Const>;
    type IterMut = QueryIter<'b, T>;

    fn mandatory_component_types() -> Vec<TypeId> {
        Vec::new()
    }

    fn lock(data: &'a SystemData<'_>) -> Self::Guard {
        data
    }

    fn item_count(_guard: &Self::Guard, _archetypes: &[ArchetypeInfo]) -> usize {
        1
    }

    fn iter(data: SystemParamData<'b, Self::Guard>) -> Self::Iter {
        QueryIter {
            data: data.guard,
            entity_pos: 0..data.item_count,
            phantom: PhantomData,
        }
    }

    fn iter_mut(data: SystemParamDataMut<'b, Self::Guard>) -> Self::IterMut {
        QueryIter {
            data: data.guard,
            entity_pos: 0..data.item_count,
            phantom: PhantomData,
        }
    }
}

struct QueryIter<'a, T> {
    #[allow(clippy::type_complexity)]
    data: &'a SystemData<'a>,
    entity_pos: Range<usize>,
    phantom: PhantomData<T>,
}

impl<'a, T> Iterator for QueryIter<'a, T>
where
    T: QueryParam,
{
    type Item = Query<'a, T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.entity_pos.next().map(|_| Query {
            data: self.data,
            phantom: PhantomData,
        })
    }
}

impl<'a, T> DoubleEndedIterator for QueryIter<'a, T>
where
    T: QueryParam,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for QueryIter<'a, T>
where
    T: QueryParam,
{
    fn len(&self) -> usize {
        self.entity_pos.end
    }
}

struct QueryGuard<'a, 'b, T>
where
    T: SystemParam<'a, 'b>,
{
    sorted_archetypes: Vec<ArchetypeInfo>,
    item_count: usize,
    guard: T::Guard,
}

impl<'a, 'b, T> QueryGuard<'a, 'b, T>
where
    'a: 'b,
    T: SystemParam<'a, 'b>,
{
    fn iter(&'b self) -> T::Iter {
        let data = SystemParamData {
            guard: &self.guard,
            sorted_archetypes: &self.sorted_archetypes,
            item_count: self.item_count,
        };
        T::iter(data)
    }
}

struct QueryGuardMut<'a, 'b, T>
where
    T: SystemParam<'a, 'b>,
{
    sorted_archetypes: Vec<ArchetypeInfo>,
    item_count: usize,
    guard: T::Guard,
}

impl<'a, 'b, T> QueryGuardMut<'a, 'b, T>
where
    'a: 'b,
    T: SystemParam<'a, 'b>,
{
    fn iter(&'b mut self) -> T::Iter {
        let data = SystemParamData {
            guard: &self.guard,
            sorted_archetypes: &self.sorted_archetypes,
            item_count: self.item_count,
        };
        T::iter(data)
    }

    fn iter_mut(&'b mut self) -> T::IterMut {
        let data = SystemParamDataMut {
            guard: &mut self.guard,
            sorted_archetypes: &self.sorted_archetypes,
            item_count: self.item_count,
        };
        T::iter_mut(data)
    }
}

trait QueryParam {}

impl<A, B> QueryParam for (A, B)
where
    A: QueryParam,
    B: QueryParam,
{
}

impl<C> QueryParam for &C where C: Any {}

impl<C> QueryParam for &mut C where C: Any {}

trait SystemParamWithLifetimes<'a, 'b> {
    type WithLifetimes: SystemParam<'a, 'b>;
}

impl<'a, 'b, C> SystemParamWithLifetimes<'a, 'b> for &C
where
    C: Any,
{
    type WithLifetimes = &'b C;
}

impl<'a, 'b, C> SystemParamWithLifetimes<'a, 'b> for &mut C
where
    C: Any,
{
    type WithLifetimes = &'b mut C;
}

impl<'a, 'b, A, B> SystemParamWithLifetimes<'a, 'b> for (A, B)
where
    'a: 'b,
    A: SystemParamWithLifetimes<'a, 'b>,
    B: SystemParamWithLifetimes<'a, 'b>,
{
    type WithLifetimes = (A::WithLifetimes, B::WithLifetimes);
}

impl<'a, 'b, T> SystemParamWithLifetimes<'a, 'b> for Query<'_, T>
where
    'a: 'b,
    T: QueryParam + SystemParamWithLifetimes<'a, 'b>,
    T::WithLifetimes: QueryParam,
    <T::WithLifetimes as SystemParam<'a, 'b>>::Const: QueryParam,
{
    type WithLifetimes = Query<'b, T::WithLifetimes>;
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn system1(a: &mut i8, archetype: Archetype) {
    println!("{}, archetype: {}", a, archetype.idx);
}

fn system2(query1: Query<'_, &mut i8>, mut query2: Query<'_, (&mut u32, &mut i64)>) {
    println!("Initial values:");
    for (a, b) in query2.lock_mut().iter() {
        println!("{}, {}", a, b);
    }
    for (a, b) in query2.lock_mut().iter_mut().rev() {
        *b += i64::from(*a);
    }
    println!("Modified values reversed:");
    for (a, b) in query2.lock().iter().rev() {
        println!("{}, {}", a, b);
    }
    println!("Other values:");
    for a in query1.lock().iter() {
        println!("{}", a);
    }
}

#[test]
fn main() {
    let components: Vec<Box<dyn Any>> = vec![
        Box::new(RwLock::new(vec![vec![1_u32, 2, 3], vec![4, 5, 6]])),
        Box::new(RwLock::new(vec![vec![7_i64, 8, 9], vec![10, 11, 12]])),
        Box::new(RwLock::new(vec![vec![13_i8, 14, 15], vec![16, 17, 18]])),
        Box::new(RwLock::new(vec![vec![19_u8, 20, 21], vec![22, 23, 24]])),
    ];
    let type_idxs = vec![
        (TypeId::of::<u32>(), 0),
        (TypeId::of::<i64>(), 1),
        (TypeId::of::<i8>(), 2),
        (TypeId::of::<u8>(), 3),
    ]
    .into_iter()
    .collect::<FxHashMap<_, _>>();
    let data = SystemData {
        components: &components,
        type_idxs: &type_idxs,
    };

    SystemRunner::new(&mut system1, &data).run();
    SystemRunner::new(&mut system2, &data).run();
}
