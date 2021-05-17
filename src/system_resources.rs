use crate::{
    ConstSystemParam, EntityBuilder, EntityMainComponent, GroupBuilder, System, SystemData,
    TupleSystemParam,
};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub struct Group<'a> {
    group_idx: NonZeroUsize,
    data: SystemData<'a>,
}

impl<'a> Group<'a> {
    pub fn replace<F>(&mut self, build_group_fn: F)
    where
        F: FnOnce(&mut GroupBuilder<'_>) + Sync + Send + 'static,
    {
        self.data
            .actions_mut()
            .mark_group_as_replaced(self.group_idx, Box::new(build_group_fn));
    }

    pub fn delete(&mut self) {
        self.data
            .actions_mut()
            .mark_group_as_deleted(self.group_idx);
    }

    pub fn create_entity<M>(&mut self, params: M::Params)
    where
        M: EntityMainComponent,
    {
        let group_idx = self.group_idx;
        self.data.actions_mut().add_entity_to_create(
            group_idx,
            Box::new(move |m| {
                let entity_idx = m.create_entity(group_idx);
                M::build(&mut EntityBuilder::new(m, entity_idx, group_idx), params);
            }),
        );
    }

    pub(crate) fn new(group_idx: NonZeroUsize, data: SystemData<'a>) -> Self {
        Self { group_idx, data }
    }
}

pub struct Entity<'a> {
    entity_idx: usize,
    data: SystemData<'a>,
}

impl<'a> Entity<'a> {
    pub fn delete(&mut self) {
        self.data
            .actions_mut()
            .mark_entity_as_deleted(self.entity_idx)
    }

    pub fn add_component<C>(&mut self, component: C)
    where
        C: Any + Sync + Send,
    {
        let entity_idx = self.entity_idx;
        self.data.actions_mut().add_component_to_add(
            entity_idx,
            Box::new(move |m| m.add_component(entity_idx, component)),
        )
    }

    pub fn delete_component<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.data
            .actions_mut()
            .mark_component_as_deleted::<C>(self.entity_idx)
    }

    pub(crate) fn new(entity_idx: usize, data: SystemData<'a>) -> Self {
        Self { entity_idx, data }
    }
}

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
            pub fn run<S>(&self, system: S) -> QueryRun<'a, S>
            where
                S: System<'b, 'c, ($($param,)*)>,
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
            pub fn run<S>(&mut self, system: S) -> QueryRun<'a, S>
            where
                S: System<'b, 'c, ($($param,)*)>,
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

pub struct QueryRun<'a, S> {
    pub data: SystemData<'a>,
    pub system: S,
    pub filtered_component_types: Vec<TypeId>,
    pub group_idx: Option<NonZeroUsize>,
}
