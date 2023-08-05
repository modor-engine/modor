use crate::storages::core::CoreStorage;
use crate::storages::systems::SystemProperties;
use crate::system_params::entity::internal::{EntityGuard, EntityGuardBorrow, EntityIter};
use crate::system_params::internal::{LockableSystemParam, Mut};
use crate::system_params::world::internal::{WorldGuard, WorldGuardBorrow, WorldStream};
use crate::systems::context::SystemContext;
use crate::{
    BuildableEntity, Component, ComponentSystems, Entity, SystemParam, SystemParamWithLifetime,
    VariableSend, VariableSync, World,
};
use std::any::Any;

/// A system parameter for performing actions on an entity.
///
/// This parameter is equivalent to the combination of [`Entity`] and [`World`] parameters.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// #[derive(Component, NoSystem)]
/// struct Velocity(f32, f32);
///
/// fn make_static(mut entity: EntityMut<'_>) {
///     entity.delete_component::<Velocity>();
/// }
/// ```
pub struct EntityMut<'a> {
    entity: Entity<'a>,
    world: World<'a>,
}

impl<'a> EntityMut<'a> {
    /// Returns the inner [`Entity`] instance.
    pub fn entity(&self) -> Entity<'_> {
        self.entity
    }

    /// Returns the inner [`World`] instance.
    pub fn world(&mut self) -> &mut World<'a> {
        &mut self.world
    }

    /// Creates a new child entity.
    ///
    /// The entity is actually created once all registered systems have been run.
    pub fn create_child<T>(
        &mut self,
        entity: impl BuildableEntity<T> + Any + VariableSync + VariableSend,
    ) {
        self.world.create_child_entity(self.entity.id(), entity);
    }

    /// Deletes the entity.
    ///
    /// The entity is actually deleted once all registered systems have been run.
    pub fn delete(&mut self) {
        self.world.delete_entity(self.entity.id());
    }

    /// Adds a component of type `C` to the entity.
    ///
    /// The component is actually added once all registered systems have been run.
    ///
    /// If the entity already has a component of type `C`, it is overwritten.
    pub fn add_component<C>(&mut self, component: C)
    where
        C: ComponentSystems,
    {
        self.world.add_component(self.entity.id(), component);
    }

    /// Deletes the component of type `C` from the entity.
    ///
    /// The component is actually deleted once all registered systems have been run.
    ///
    /// If the entity does not have a component of type `C`, nothing is done.
    pub fn delete_component<C>(&mut self)
    where
        C: Component,
    {
        self.world.delete_component::<C>(self.entity.id());
    }
}

impl<'a> SystemParamWithLifetime<'a> for EntityMut<'_> {
    type Param = EntityMut<'a>;
    type Guard = (EntityGuard<'a>, WorldGuard<'a>);
    type GuardBorrow = (EntityGuardBorrow<'a>, WorldGuardBorrow<'a>);
    type Stream = (EntityIter<'a>, WorldStream<'a>);
}

impl SystemParam for EntityMut<'_> {
    type Filter = ();
    type InnerTuple = ();

    fn properties(_core: &mut CoreStorage) -> SystemProperties {
        SystemProperties {
            component_types: vec![],
            can_update: true,
            mutation_component_type_idxs: vec![],
        }
    }

    fn lock(context: SystemContext<'_>) -> <Self as SystemParamWithLifetime<'_>>::Guard {
        (EntityGuard::new(context), WorldGuard::new(context))
    }

    fn borrow_guard<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::Guard,
    ) -> <Self as SystemParamWithLifetime<'a>>::GuardBorrow
    where
        'b: 'a,
    {
        (guard.0.borrow(), guard.1.borrow())
    }

    fn stream<'a, 'b>(
        guard: &'a mut <Self as SystemParamWithLifetime<'b>>::GuardBorrow,
    ) -> <Self as SystemParamWithLifetime<'a>>::Stream
    where
        'b: 'a,
    {
        (EntityIter::new(&guard.0), WorldStream::new(&guard.1))
    }

    #[inline]
    fn stream_next<'a, 'b>(
        stream: &'a mut <Self as SystemParamWithLifetime<'b>>::Stream,
    ) -> Option<<Self as SystemParamWithLifetime<'a>>::Param>
    where
        'b: 'a,
    {
        if let (Some(entity), Some(world)) = (stream.0.next(), stream.1.next()) {
            Some(EntityMut { entity, world })
        } else {
            None
        }
    }
}

impl LockableSystemParam for EntityMut<'_> {
    type LockedType = World<'static>;
    type Mutability = Mut;
}
