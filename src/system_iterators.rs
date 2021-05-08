use crate::{QueryMut, TupleSystemParam};
use std::any::Any;
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

pub struct QueryMutIterator<'a, T>(QueryMut<'a, T>)
where
    T: TupleSystemParam;

impl<'a, T> QueryMutIterator<'a, T>
where
    T: TupleSystemParam,
{
    pub(crate) fn new(iter: QueryMut<'a, T>) -> Self {
        QueryMutIterator(iter)
    }
}

impl<'a, T> Iterator for QueryMutIterator<'a, T>
where
    T: TupleSystemParam,
{
    type Item = QueryMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.0.clone())
    }
}
