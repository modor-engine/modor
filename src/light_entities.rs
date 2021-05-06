use crate::internal::main::MainFacade;
use crate::{Built, Entity, EntityBuilder};
use std::any::Any;
use std::marker::PhantomData;
use std::num::NonZeroUsize;

pub trait LightEntity: Sized + Any + Sync + Send {
    type LightParams: Any;

    fn build(builder: &mut LightEntityBuilder<'_, Self>, params: Self::LightParams);
}

impl<E> Entity for E
where
    E: LightEntity,
{
    type Params = <Self as LightEntity>::LightParams;

    fn build(builder: &mut EntityBuilder<'_, Self>, params: Self::Params) -> Built {
        Self::build(&mut builder.light_builder(), params);
        Built::new()
    }
}

pub struct LightEntityBuilder<'a, E> {
    ecs: &'a mut MainFacade,
    entity_idx: usize,
    group_idx: NonZeroUsize,
    phantom: PhantomData<E>,
}

impl<'a, E> LightEntityBuilder<'a, E>
where
    E: Entity + LightEntity,
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

    pub(crate) fn new(ecs: &'a mut MainFacade, entity_idx: usize, group_idx: NonZeroUsize) -> Self {
        Self {
            ecs,
            entity_idx,
            group_idx,
            phantom: PhantomData,
        }
    }
}
