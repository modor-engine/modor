use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::SystemBuilder;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub trait EntityMainComponent: Sized + Any + Sync + Send {
    type Params: Any + Sync + Send;

    fn build(builder: &mut EntityBuilder<'_, Self>, params: Self::Params) -> Built;

    #[allow(unused_variables)]
    fn on_update(runner: &mut EntityRunner<'_, Self>) {}
}

pub struct EntityBuilder<'a, M> {
    main: &'a mut MainFacade,
    entity_idx: usize,
    group_idx: NonZeroUsize,
    phantom: PhantomData<M>,
}

impl<'a, M> EntityBuilder<'a, M>
where
    M: EntityMainComponent,
{
    pub fn inherit_from<P>(&mut self, params: P::Params) -> &mut Self
    where
        P: EntityMainComponent,
    {
        let mut entity_builder = EntityBuilder::new(self.main, self.entity_idx, self.group_idx);
        P::build(&mut entity_builder, params);
        self
    }

    pub fn with<C>(&mut self, component: C) -> &mut Self
    where
        C: Any + Sync + Send,
    {
        self.main.add_component(self.entity_idx, component);
        self
    }

    pub fn with_self(&mut self, entity: M) -> Built {
        self.with(entity);
        let new_type = self.main.add_entity_main_component::<M>();
        if new_type {
            M::on_update(&mut EntityRunner::new(self.main));
        }
        Built::new()
    }

    pub(crate) fn new(
        main: &'a mut MainFacade,
        entity_idx: usize,
        group_idx: NonZeroUsize,
    ) -> Self {
        Self {
            main,
            entity_idx,
            group_idx,
            phantom: PhantomData,
        }
    }
}

pub struct EntityRunner<'a, M> {
    main: &'a mut MainFacade,
    phantom: PhantomData<M>,
}

impl<'a, M> EntityRunner<'a, M>
where
    M: EntityMainComponent,
{
    pub fn run(&mut self, system: SystemBuilder) -> &mut Self {
        let entity_type = Some(TypeId::of::<M>());
        let system = SystemDetails::new(
            system.wrapper,
            system.component_types,
            entity_type,
            system.actions,
        );
        self.main.add_system(None, system);
        self
    }

    fn new(ecs: &'a mut MainFacade) -> Self {
        Self {
            main: ecs,
            phantom: PhantomData,
        }
    }
}

pub struct Built(PhantomData<()>);

impl Built {
    fn new() -> Self {
        Self(PhantomData)
    }
}

#[cfg(test)]
mod entity_builder_tests {
    use super::*;

    assert_impl_all!(EntityBuilder<'_, String>: Send);
    assert_not_impl_any!(EntityBuilder<'_, String>: Clone);
}

#[cfg(test)]
mod entity_runner_tests {
    use super::*;

    assert_impl_all!(EntityRunner<'_, String>: Send);
    assert_not_impl_any!(EntityRunner<'_, String>: Clone);
}

#[cfg(test)]
mod built_tests {
    use super::*;

    assert_impl_all!(Built: Sync, Send);
    assert_not_impl_any!(Built: Clone);
}
