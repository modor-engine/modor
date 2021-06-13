use crate::{
    ConstSystemParam, EntityBuilder, EntityMainComponent, GroupBuilder, System, SystemData,
    TupleSystemParam,
};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// Group in which an entity queried by a system is located.
///
/// This system parameter can only be specified for iterative systems (see documentation of
/// the [`system!`](crate::system) macro for more information about types of system).
///
/// # Examples
///
/// ```rust
/// # use modor::{Application, Group, system};
/// #
/// Application::new()
///     .on_update(system!(run_system))
///     .update();
///
/// fn run_system(string: &String, mut group: Group<'_>) {
///     if string == "group to delete" {
///         group.delete();
///     }
/// }
/// ```
pub struct Group<'a> {
    group_idx: NonZeroUsize,
    data: SystemData<'a>,
}

impl<'a> Group<'a> {
    /// Replace the group by another one.
    ///
    /// The actual replacement is done at the end of the application update, once all systems have
    /// been run.<br>
    /// If the group is deleted before the end of the application update, the replacement is
    /// canceled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Group, EntityMainComponent, Built, EntityBuilder, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut group: Group<'_>) {
    ///     if string == "group to replace" {
    ///         group.replace(|builder| {
    ///             builder
    ///                 .with_entity::<Button>("Ok")
    ///                 .with_entity::<Button>("Cancel");
    ///         });
    ///     }
    /// }
    /// #
    /// # struct Button;
    /// #
    /// # impl EntityMainComponent for Button {
    /// #     type Data = &'static str;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>,data: Self::Data) -> Built {
    /// #         builder.with_self(Self)
    /// #     }
    /// # }
    /// ```
    pub fn replace<F>(&mut self, build_group_fn: F)
    where
        F: FnOnce(&mut GroupBuilder<'_>) + Sync + Send + 'static,
    {
        self.data
            .actions_mut()
            .replace_group(self.group_idx, Box::new(build_group_fn));
    }

    /// Delete the group.
    ///
    /// The actual deletion is done at the end of the application update, once all systems have
    /// been run.<br>
    /// All entities contained in the group are deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Group, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut group: Group<'_>) {
    ///     if string == "group to delete" {
    ///         group.delete();
    ///     }
    /// }
    /// ```
    pub fn delete(&mut self) {
        self.data.actions_mut().delete_group(self.group_idx);
    }

    /// Create an entity in the group.
    ///
    /// The actual creation is done at the end of the application update, once all systems have
    /// been run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Group, EntityMainComponent, Built, EntityBuilder, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut group: Group<'_>) {
    ///     if string == "entity to create" {
    ///         group.create_entity::<Button>("Are you sure ?");
    ///         group.create_entity::<Button>("Yes");
    ///         group.create_entity::<Button>("No");
    ///     }
    /// }
    /// #
    /// # struct Button;
    /// #
    /// # impl EntityMainComponent for Button {
    /// #     type Data = &'static str;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>,data: Self::Data) -> Built {
    /// #         builder.with_self(Self)
    /// #     }
    /// # }
    /// ```
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

/// Entity queried by a system.
///
/// This system parameter can only be specified for iterative systems (see documentation of
/// the [`system!`](crate::system) macro for more information about types of system).
///
/// # Examples
///
/// ```rust
/// # use modor::{Application, Entity, system};
/// #
/// Application::new()
///     .on_update(system!(run_system))
///     .update();
///
/// fn run_system(string: &String, mut entity: Entity<'_>) {
///     if string == "entity to delete" {
///         entity.delete();
///     }
/// }
/// ```
pub struct Entity<'a> {
    entity_idx: usize,
    data: SystemData<'a>,
}

impl<'a> Entity<'a> {
    /// Delete the entity.
    ///
    /// The actual deletion is done at the end of the application update, once all systems have
    /// been run.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Entity, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut entity: Entity<'_>) {
    ///     if string == "entity to delete" {
    ///         entity.delete();
    ///     }
    /// }
    /// ```
    pub fn delete(&mut self) {
        self.data.actions_mut().delete_entity(self.entity_idx)
    }

    /// Add a component to the entity.
    ///
    /// The actual adding is done at the end of the application update, once all systems have
    /// been run.<br>
    /// If a component of the type `C` already exists for the entity, the existing component is
    /// overwritten.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Entity, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut entity: Entity<'_>) {
    ///     if string == "component to add" {
    ///         entity.add_component(Position(78.4, 65.2));
    ///     }
    /// }
    ///
    /// struct Position(f32, f32);
    /// ```
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

    /// Delete a component from the entity.
    ///
    /// The actual deletion is done at the end of the application update, once all systems have
    /// been run.<br>
    /// If no component of type `C` exists for the entity, nothing is done.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Entity, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system(string: &String, mut entity: Entity<'_>) {
    ///     if string == "component to delete" {
    ///         entity.delete_component::<Position>();
    ///     }
    /// }
    ///
    /// struct Position(f32, f32);
    /// ```
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

/// Query runnable during the execution of a system.
///
/// The query can be run using macros [`for_each!`](crate::for_each!) and
/// [`for_each_mut!`](crate::for_each_mut!).
///
/// # Examples
///
/// ```rust
/// # use modor::{Application, Query, system, for_each_mut};
/// #
/// Application::new()
///     .on_update(system!(run_system))
///     .update();
///
/// fn run_system<'a>(mut query: Query<'a, (&'a u32, Option<&'a mut String>)>) {
///     for_each_mut!(query, |id: &u32, string: Option<&mut String>| {
///         if let Some(string) = string {
///             *string = format!("id: {}", id);
///         }
///     });
/// }
/// ```
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
    /// Add a component type filter to the query.
    ///
    /// It has the same effect as the components already in system parameters, i.e. it filters
    /// the entities on which the query will iterate according to its component types.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Query, system};
    /// #
    /// Application::new()
    ///     .on_update(system!(run_system))
    ///     .update();
    ///
    /// fn run_system<'a>(mut query: Query<'a, (&'a mut u32, &'a String)>) {
    ///     query.filter::<i64>();
    ///     // now if query is used, iterated entities must have an `i64` component in addition to
    ///     // `u32` and `String` components
    /// }
    /// ```
    pub fn filter<C>(&mut self)
    where
        C: Any,
    {
        self.filtered_component_types.push(TypeId::of::<C>());
    }

    /// Indicate the query will iterate on any group.
    ///
    /// This method takes effect only in group systems.<br>
    /// By default, `Query` parameters from group systems iterates only on entities in the
    /// associated group.
    /// <br>
    /// To be able to iterate on entities from any group, this method must be called.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, Query, system};
    /// #
    /// Application::new()
    ///     .with_group(|builder| {
    ///         builder.on_update(system!(run_system));
    ///     })
    ///     .update();
    ///
    /// fn run_system<'a>(mut query: Query<'a, (&'a mut u32, &'a String)>) {
    ///     query.unlock_groups();
    ///     // now the query will iterate on entities from any group
    /// }
    /// ```
    pub fn unlock_groups(&mut self) {
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
            #[doc(hidden)]
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

            #[doc(hidden)]
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

#[doc(hidden)]
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
