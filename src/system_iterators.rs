use crate::{Entity, Group, Query, SystemData, TupleSystemParam};
use std::any::Any;
use std::num::NonZeroUsize;
use std::slice::{Iter, IterMut};

#[doc(hidden)]
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

#[doc(hidden)]
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

#[doc(hidden)]
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

#[doc(hidden)]
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

#[doc(hidden)]
pub struct QueryIter<'a, T>(Query<'a, T>)
where
    T: TupleSystemParam;

impl<'a, T> QueryIter<'a, T>
where
    T: TupleSystemParam,
{
    pub(crate) fn new(iter: Query<'a, T>) -> Self {
        QueryIter(iter)
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
}

#[cfg(test)]
mod option_component_mut_iter_tests {
    use super::*;

    assert_impl_all!(OptionComponentMutIter<'_, u32>: Sync, Send);
    assert_not_impl_any!(OptionComponentMutIter<'_, u32>: Clone);
}

#[cfg(test)]
mod group_iter_tests {
    use super::*;

    assert_impl_all!(GroupIter<'_>: Sync, Send);
    assert_not_impl_any!(GroupIter<'_>: Clone);
}

#[cfg(test)]
mod entity_iter_tests {
    use super::*;

    assert_impl_all!(EntityIter<'_>: Sync, Send);
    assert_not_impl_any!(EntityIter<'_>: Clone);
}

#[cfg(test)]
mod query_iter_tests {
    use super::*;

    assert_impl_all!(QueryIter<'_, (&u32, )>: Sync, Send);
    assert_not_impl_any!(QueryIter<'_, (&u32, )>: Clone);
}
