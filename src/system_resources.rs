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
            .replace_group(self.group_idx, Box::new(build_group_fn));
    }

    pub fn delete(&mut self) {
        self.data.actions_mut().delete_group(self.group_idx);
    }

    pub fn create_entity<M>(&mut self, data: M::Data)
    where
        M: EntityMainComponent,
    {
        let group_idx = self.group_idx;
        self.data.actions_mut().create_entity(
            group_idx,
            Box::new(move |m| {
                let entity_idx = m.create_entity(group_idx);
                M::build(&mut EntityBuilder::new(m, entity_idx, group_idx), data);
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
        self.data.actions_mut().delete_entity(self.entity_idx)
    }

    pub fn add_component<C>(&mut self, component: C)
    where
        C: Any + Sync + Send,
    {
        let entity_idx = self.entity_idx;
        self.data.actions_mut().add_component(
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
            .delete_component::<C>(self.entity_idx)
    }

    pub(crate) fn new(entity_idx: usize, data: SystemData<'a>) -> Self {
        Self { entity_idx, data }
    }
}

pub struct Query<'a, T>
where
    T: TupleSystemParam,
{
    data: SystemData<'a>,
    filtered_component_types: Vec<TypeId>,
    group_idx: Option<NonZeroUsize>,
    phantom: PhantomData<T>,
}

impl<'a, T> Query<'a, T>
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

    pub(crate) fn duplicate(&self) -> Self {
        Self {
            data: self.data.clone(),
            filtered_component_types: self.filtered_component_types.clone(),
            group_idx: self.group_idx,
            phantom: PhantomData,
        }
    }
}

impl<'a, T> Clone for Query<'a, T>
where
    T: TupleSystemParam + ConstSystemParam,
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

macro_rules! impl_query_run {
    ($($params:ident),*) => {
        impl<'a, 'b, 'c $(,$params)*> Query<'a, ($($params,)*)>
        where
            ($($params,)*): TupleSystemParam,
        {
            pub fn run<S>(&self, system: S) -> QueryRun<'a, S>
            where
                S: System<'b, 'c, ($($params,)*)>,
                ($($params,)*): ConstSystemParam,
            {
                QueryRun {
                    data: self.data.clone(),
                    system,
                    filtered_component_types: self.filtered_component_types.clone(),
                    group_idx: self.group_idx,
                }
            }

            pub fn run_mut<S>(&mut self, system: S) -> QueryRun<'a, S>
            where
                S: System<'b, 'c, ($($params,)*)>,
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

pub struct QueryRun<'a, S> {
    pub data: SystemData<'a>,
    pub system: S,
    pub filtered_component_types: Vec<TypeId>,
    pub group_idx: Option<NonZeroUsize>,
}

#[cfg(test)]
mod group_tests {
    use super::*;

    assert_impl_all!(Group<'_>: Sync, Send);
    assert_not_impl_any!(Group<'_>: Clone);
}

#[cfg(test)]
mod entity_tests {
    use super::*;

    assert_impl_all!(Entity<'_>: Sync, Send);
    assert_not_impl_any!(Entity<'_>: Clone);
}

#[cfg(test)]
mod query_tests {
    use super::*;

    assert_impl_all!(Query<'_, (&u32,)>: Sync, Send, Clone);
    assert_impl_all!(Query<'_, (&mut u32,)>: Sync, Send);
    assert_not_impl_any!(Query<'_, (&mut u32,)>: Clone);
}

#[cfg(test)]
mod query_run_tests {
    use super::*;

    assert_impl_all!(QueryRun<'_, fn(&u32,)>: Sync, Send);
    assert_not_impl_any!(QueryRun<'_, fn(&u32,)>: Clone);
}
