use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemInfo;
use crate::{LightEntity, LightEntityBuilder, SystemBuilder};
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub trait Entity: Sized + Any + Sync + Send {
    type Params: Any;

    fn build(builder: &mut EntityBuilder<'_, Self>, params: Self::Params) -> Built;

    #[allow(unused_variables)]
    fn on_update(runner: &mut EntityRunner<'_, Self>) {}
}

pub struct EntityBuilder<'a, E> {
    ecs: &'a mut MainFacade,
    entity_idx: usize,
    group_idx: NonZeroUsize,
    phantom: PhantomData<E>,
}

impl<'a, E> EntityBuilder<'a, E>
where
    E: Entity,
{
    pub fn inherit_from<P>(&mut self, params: P::Params) -> &mut Self
    where
        P: Entity,
    {
        let mut entity_builder = EntityBuilder::new(self.ecs, self.entity_idx, self.group_idx);
        P::build(&mut entity_builder, params);
        self
    }

    pub fn with<C>(&mut self, component: C) -> &mut Self
    where
        C: Any + Sync + Send,
    {
        self.ecs.add_component(self.entity_idx, component);
        self
    }

    pub fn with_self(&mut self, entity: E) -> Built {
        self.with(entity);
        if self.ecs.add_entity_type(self.group_idx, TypeId::of::<E>()) {
            E::on_update(&mut EntityRunner::new(self.ecs, self.group_idx));
        }
        Built::new()
    }

    pub(crate) fn new(ecs: &'a mut MainFacade, entity_idx: usize, group_idx: NonZeroUsize) -> Self {
        Self {
            ecs,
            entity_idx,
            group_idx,
            phantom: PhantomData,
        }
    }

    pub(crate) fn light_builder(&mut self) -> LightEntityBuilder<'_, E>
    where
        E: LightEntity,
    {
        LightEntityBuilder::new(self.ecs, self.entity_idx, self.group_idx)
    }
}

pub struct EntityRunner<'a, E> {
    ecs: &'a mut MainFacade,
    group_idx: NonZeroUsize,
    phantom: PhantomData<E>,
}

impl<'a, E> EntityRunner<'a, E>
where
    E: Entity,
{
    pub fn run(&mut self, system: SystemBuilder) -> &mut Self {
        let entity_type = Some(TypeId::of::<E>());
        let system = SystemInfo::new(
            system.wrapper,
            system.component_types,
            entity_type,
            system.group_actions,
        );
        self.ecs.add_system(Some(self.group_idx), system);
        self
    }

    pub(crate) fn new(ecs: &'a mut MainFacade, group_idx: NonZeroUsize) -> Self {
        Self {
            ecs,
            group_idx,
            phantom: PhantomData,
        }
    }
}

pub struct Built(PhantomData<()>);

impl Built {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}
