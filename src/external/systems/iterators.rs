use crate::{Entity, Group, Query, SystemData, TupleSystemParam};
use std::any::Any;
use std::num::NonZeroUsize;
use std::slice::{Iter, IterMut};

pub struct OptionComponentIter<'a, C>(Option<Iter<'a, C>>)
where
    C: Any;

impl<'a, C> OptionComponentIter<'a, C>
where
    C: Any,
{
    pub(crate) fn new(iter: Option<Iter<'a, C>>) -> Self {
        OptionComponentIter(iter)
    }
}

impl<'a, C> Iterator for OptionComponentIter<'a, C>
where
    C: Any,
{
    type Item = Option<&'a C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .as_mut()
            .map_or(Some(None), |iter| iter.next().map(Some))
    }
}

pub struct OptionComponentMutIter<'a, C>(Option<IterMut<'a, C>>)
where
    C: Any;

impl<'a, C> OptionComponentMutIter<'a, C>
where
    C: Any,
{
    pub(crate) fn new(iter: Option<IterMut<'a, C>>) -> Self {
        OptionComponentMutIter(iter)
    }
}

impl<'a, C> Iterator for OptionComponentMutIter<'a, C>
where
    C: Any,
{
    type Item = Option<&'a mut C>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .as_mut()
            .map_or(Some(None), |iter| iter.next().map(Some))
    }
}

pub struct GroupIter<'a> {
    group_idx: NonZeroUsize,
    data: SystemData<'a>,
}

impl<'a> GroupIter<'a> {
    pub(crate) fn new(group_idx: NonZeroUsize, data: SystemData<'a>) -> Self {
        Self { group_idx, data }
    }
}

impl<'a> Iterator for GroupIter<'a> {
    type Item = Group<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Group::new(self.group_idx, self.data.clone()))
    }
}

pub struct EntityIter<'a> {
    entity_idxs: Iter<'a, usize>,
    data: SystemData<'a>,
}

impl<'a> EntityIter<'a> {
    pub(crate) fn new(entity_idxs: Iter<'a, usize>, data: SystemData<'a>) -> Self {
        Self { entity_idxs, data }
    }
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = Entity<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.entity_idxs
            .next()
            .map(|&i| Entity::new(i, self.data.clone()))
    }
}

pub struct QueryIter<'a, T>(Query<'a, T>)
where
    T: TupleSystemParam;

impl<'a, T> QueryIter<'a, T>
where
    T: TupleSystemParam,
{
    pub(crate) fn new(query: Query<'a, T>) -> Self {
        QueryIter(query)
    }
}

impl<'a, T> Iterator for QueryIter<'a, T>
where
    T: TupleSystemParam,
{
    type Item = Query<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.duplicate())
    }
}

#[cfg(test)]
mod option_component_iter_tests {
    use super::*;

    assert_impl_all!(OptionComponentIter<'_, u32>: Sync, Send);
    assert_not_impl_any!(OptionComponentIter<'_, u32>: Clone);

    #[test]
    fn create_present_iter() {
        let components = vec![10, 20];
        let iter = Some(components.iter());

        let component_iter = OptionComponentIter::<u32>::new(iter);

        assert_iter!(component_iter, [Some(&10), Some(&20)]);
    }

    #[test]
    fn create_missing_iter() {
        let iter = None;

        let mut component_iter = OptionComponentIter::<u32>::new(iter);

        assert_eq!(component_iter.next(), Some(None));
        assert_eq!(component_iter.next(), Some(None));
    }
}

#[cfg(test)]
mod option_component_mut_iter_tests {
    use super::*;

    assert_impl_all!(OptionComponentMutIter<'_, u32>: Sync, Send);
    assert_not_impl_any!(OptionComponentMutIter<'_, u32>: Clone);

    #[test]
    fn create_present_iter() {
        let mut components = vec![10, 20];
        let iter = Some(components.iter_mut());

        let component_iter = OptionComponentMutIter::<u32>::new(iter);

        assert_iter!(component_iter, [Some(&mut 10), Some(&mut 20)]);
    }

    #[test]
    fn create_missing_iter() {
        let iter = None;

        let mut component_iter = OptionComponentMutIter::<u32>::new(iter);

        assert_eq!(component_iter.next(), Some(None));
        assert_eq!(component_iter.next(), Some(None));
    }
}

#[cfg(test)]
mod group_iter_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemOnceBuilder;

    assert_impl_all!(GroupIter<'_>: Sync, Send);
    assert_not_impl_any!(GroupIter<'_>: Clone);

    #[test]
    fn create_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity_idx = main.create_entity(group_idx);
        main.add_component(entity_idx, 10_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut group_iter = GroupIter::new(group_idx, data.clone());

            group_iter.next().unwrap().delete();
        }));
        main.apply_system_actions();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            assert_eq!(data.entity_idxs(0), []);
        }));
    }
}

#[cfg(test)]
mod entity_iter_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemOnceBuilder;

    assert_impl_all!(EntityIter<'_>: Sync, Send);
    assert_not_impl_any!(EntityIter<'_>: Clone);

    #[test]
    fn create_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        let entity3_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.add_component(entity3_idx, 30_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let components = &[0, 1, 2];
            let mut entity_iter = EntityIter::new(components.iter(), data.clone());

            entity_iter.next().unwrap().delete();
            assert!(entity_iter.next().is_some());
            entity_iter.next().unwrap().delete();
        }));
        main.apply_system_actions();
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            assert_eq!(data.entity_idxs(0), [1]);
        }));
    }
}

#[cfg(test)]
mod query_iter_tests {
    use super::*;
    use crate::internal::main::MainFacade;
    use crate::SystemOnceBuilder;
    use std::any::TypeId;

    assert_impl_all!(QueryIter<'_, (&u32, )>: Sync, Send);
    assert_not_impl_any!(QueryIter<'_, (&u32, )>: Clone);

    #[test]
    fn create_iter() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity1_idx = main.create_entity(group_idx);
        let entity2_idx = main.create_entity(group_idx);
        main.add_component(entity1_idx, 10_u32);
        main.add_component(entity2_idx, 20_u32);
        main.run_system_once(SystemOnceBuilder::new(|data, _| {
            let mut query = Query::<(&u32,)>::new(Some(group_idx), data.clone());
            query.filter::<i64>();

            let mut query_iter = QueryIter::new(query);

            let query = query_iter.next().unwrap();
            let query_run = query.run(|_: &u32| ());
            assert_eq!(query_run.group_idx, Some(group_idx));
            assert_eq!(query_run.filtered_component_types, [TypeId::of::<i64>()]);
        }));
    }
}
