use crate::{ConstSystemParam, GroupBuilder, System, SystemData, TupleSystemParam};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub struct Query<'a, T>
where
    T: ConstSystemParam + TupleSystemParam,
{
    data: SystemData<'a>,
    filtered_component_types: Vec<TypeId>,
    group_idx: Option<NonZeroUsize>,
    phantom: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
where
    T: ConstSystemParam + TupleSystemParam,
{
    pub fn add_component<C>(&mut self)
    where
        C: Any,
    {
        self.filtered_component_types.push(TypeId::of::<C>());
    }

    pub fn accept_any_group(&mut self) {
        self.group_idx = None;
    }

    pub(crate) fn new(data: SystemData<'a>, group_idx: Option<NonZeroUsize>) -> Self {
        Self {
            data,
            filtered_component_types: Vec::new(),
            group_idx,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_query_run {
    ($($param:ident),*) => {
        impl<'a, 'b, 'c $(,$param)*> Query<'a, ($($param,)*)>
        where
            ($($param,)*): ConstSystemParam + TupleSystemParam,
        {
            pub fn run<SYS>(&self, system: SYS) -> QueryRun<'a, SYS>
            where
                SYS: System<'b, 'c, ($($param,)*)>,
            {
                QueryRun {
                    data: self.data.clone(),
                    system,
                    filtered_component_types: self.filtered_component_types.clone(),
                    group_idx: self.group_idx,
                }
            }
        }
    };
}

impl_query_run!();
run_for_tuples!(impl_query_run);

impl<'a, T> Clone for Query<'a, T>
where
    T: ConstSystemParam + TupleSystemParam,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            filtered_component_types: self.filtered_component_types.clone(),
            group_idx: self.group_idx,
            phantom: PhantomData,
        }
    }
}

pub struct QueryMut<'a, T>
where
    T: TupleSystemParam,
{
    data: SystemData<'a>,
    filtered_component_types: Vec<TypeId>,
    group_idx: Option<NonZeroUsize>,
    phantom: PhantomData<T>,
}

impl<'a, T> QueryMut<'a, T>
where
    T: TupleSystemParam,
{
    pub fn add_component<C>(&mut self)
    where
        C: Any,
    {
        self.filtered_component_types.push(TypeId::of::<C>());
    }

    pub fn accept_any_group(&mut self) {
        self.group_idx = None;
    }

    pub(crate) fn new(data: SystemData<'a>, group_idx: Option<NonZeroUsize>) -> Self {
        Self {
            data,
            filtered_component_types: Vec::new(),
            group_idx,
            phantom: PhantomData,
        }
    }

    pub(crate) fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            filtered_component_types: self.filtered_component_types.clone(),
            group_idx: self.group_idx,
            phantom: PhantomData,
        }
    }
}

macro_rules! impl_query_mut_run {
    ($($param:ident),*) => {
        impl<'a, 'b, 'c $(,$param)*> QueryMut<'a, ($($param,)*)>
        where
            ($($param,)*): TupleSystemParam,
        {
            pub fn run<SYS>(&mut self, system: SYS) -> QueryRun<'a, SYS>
            where
                SYS: System<'b, 'c, ($($param,)*)>,
            {
                QueryRun {
                    data: self.data.clone(),
                    system,
                    filtered_component_types: self.filtered_component_types.clone(),
                    group_idx: self.group_idx,
                }
            }
        }
    };
}

impl_query_mut_run!();
run_for_tuples!(impl_query_mut_run);

pub struct QueryRun<'a, SYS> {
    pub data: SystemData<'a>,
    pub system: SYS,
    pub filtered_component_types: Vec<TypeId>,
    pub group_idx: Option<NonZeroUsize>,
}

// TODO: add checks (can only be here for foreach systems)
#[derive(Clone)]
pub struct Group<'a> {
    group_idx: NonZeroUsize,
    data: SystemData<'a>,
}

// TODO: can be called many times, issue ? => Many make unusable Group after deletion
// TODO: if it's the case, handle Group and Option<Group> types as SystemParam
// TODO: also if it's the case, remove Clone impl of Group + methods consume the object
// TODO: or maybe only last operation taken into account, but is it safe ?
impl<'a> Group<'a> {
    pub(crate) fn new(group_idx: NonZeroUsize, data: SystemData<'a>) -> Self {
        Self { group_idx, data }
    }

    pub fn replace<F>(&mut self, build_group_fn: F)
    where
        F: FnOnce(&mut GroupBuilder<'_>) + Sync + Send + 'static,
    {
        self.data
            .group_actions_mut()
            .mark_group_as_replaced(self.group_idx, build_group_fn);
    }

    pub fn delete(&mut self) {
        self.data
            .group_actions_mut()
            .mark_group_as_deleted(self.group_idx);
    }
}
