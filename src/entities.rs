use crate::entities::internal::{AddedComponents, ComponentAdd, StorageWrapper};
use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::archetypes::{ArchetypeIdx, EntityLocation};
use crate::storages::core::CoreStorage;
use crate::{Action, ActionConstraint, SystemBuilder};
use std::any::{Any, TypeId};
use std::marker::PhantomData;

/// A type that indicates the entity has been built.
pub struct Built(EntityLocation);

/// A trait for defining the main component of an entity type.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// # struct Position(f32, f32);
/// # struct Velocity(f32, f32);
/// #
/// struct Object {
///     name: String,
/// }
///
/// impl EntityMainComponent for Object {
///     type Data = String;
///
///     fn build(builder: EntityBuilder<'_, Self>, name: Self::Data) -> Built {
///         builder
///             .with(Position(0., 0.))
///             .with(Velocity(1., 2.))
///             .with_self(Self { name })
///     }
///
///     fn on_update(runner: EntityRunner<'_, Self>) {
///         runner
///             .run(system!(Self::update_position))
///             .run(system!(Self::update_state));
///     }
/// }
///
/// impl Object {
///     fn update_position(&self, position: &mut Position, velocity: &Velocity) {
///         position.0 += velocity.0;
///         position.1 += velocity.1;
///         println!("New position of '{}': ({}, {})", self.name, position.0, position.1);
///     }
///
///     fn update_state(&self, position: &Position, entity: Entity<'_>, mut world: World<'_>) {
///         if position.0 > 10. {
///             world.delete_entity(entity.id());
///             println!("'{}' has been deleted", self.name);
///         }
///     }
/// }
/// ```
pub trait EntityMainComponent: Sized + Any + Sync + Send {
    /// The type of the data used to build the entity.
    type Data: Any + Sync + Send;

    /// Builds the entity.
    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built;

    /// Defines systems to run during update.
    ///
    /// The systems are only run when a component of type `Self` exists in at least one entity.
    #[allow(unused_variables)]
    fn on_update(runner: EntityRunner<'_, Self>) {}
}

/// A builder for defining the components of an entity.
///
/// # Examples
///
/// See [`EntityMainComponent`](crate::EntityMainComponent).
pub struct EntityBuilder<'a, E, A = ()> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) src_location: Option<EntityLocation>,
    pub(crate) dst_archetype_idx: ArchetypeIdx,
    pub(crate) added_components: A,
    pub(crate) phantom: PhantomData<E>,
}

impl<'a, E, A> EntityBuilder<'a, E, A>
where
    E: EntityMainComponent, // TODO: delete to simplify tests if not necessary
    A: ComponentAdd,
{
    /// Inherits from an entity with main component type `P` and building data `data`.
    ///
    /// Components and systems of the parent entity are added to the built entity.
    ///
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the existing one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    pub fn inherit_from<P>(self, data: P::Data) -> EntityBuilder<'a, E, ()>
    where
        P: EntityMainComponent,
    {
        let (core, location) = self.build();
        let built = P::build(
            EntityBuilder {
                core,
                src_location: Some(location),
                dst_archetype_idx: location.idx,
                added_components: (),
                phantom: PhantomData,
            },
            data,
        );
        EntityBuilder {
            core,
            src_location: Some(built.0),
            dst_archetype_idx: built.0.idx,
            added_components: (),
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C`.
    ///
    /// If a component of type `C` already exists, it is overwritten.
    pub fn with<C>(self, component: C) -> EntityBuilder<'a, E, AddedComponents<C, A>>
    where
        C: Any + Sync + Send,
    {
        let (type_idx, archetype_idx) = self.core.add_component_type::<C>(self.dst_archetype_idx);
        EntityBuilder {
            core: self.core,
            src_location: self.src_location,
            dst_archetype_idx: archetype_idx,
            added_components: AddedComponents {
                component,
                type_idx,
                is_added: true,
                other_components: self.added_components,
            },
            phantom: PhantomData,
        }
    }

    /// Adds a component of type `C` only if `condition` equals `true`.
    ///
    /// If `condition` equals `true` and a component of type `C` already exists, it is overwritten.
    pub fn with_if<C>(
        self,
        component: C,
        condition: bool,
    ) -> EntityBuilder<'a, E, AddedComponents<C, A>>
    where
        C: Any + Sync + Send,
    {
        if condition {
            self.with(component)
        } else {
            let (type_idx, _) = self.core.add_component_type::<C>(self.dst_archetype_idx);
            EntityBuilder {
                core: self.core,
                src_location: self.src_location,
                dst_archetype_idx: self.dst_archetype_idx,
                added_components: AddedComponents {
                    component,
                    type_idx,
                    is_added: false,
                    other_components: self.added_components,
                },
                phantom: PhantomData,
            }
        }
    }

    /// Add the main component of the entity.
    pub fn with_self(self, entity: E) -> Built {
        let builder = self.with(entity);
        let components = builder.core.components();
        if !components.is_entity_type::<E>() {
            builder.core.add_entity_type::<E>();
            E::on_update(EntityRunner {
                core: builder.core,
                phantom: PhantomData,
            });
        }
        let (_, location) = builder.build();
        Built(location)
    }

    fn build(self) -> (&'a mut CoreStorage, EntityLocation) {
        let location = if let Some(src_location) = self.src_location {
            self.core.move_entity(src_location, self.dst_archetype_idx)
        } else {
            self.core.create_entity(self.dst_archetype_idx).1
        };
        let mut storage = StorageWrapper(self.core);
        self.added_components.add(&mut storage, location);
        (self.core, location)
    }
}

/// A type for defining the first system of an entity.
///
/// Cyclic dependencies between systems are detected at compile time.
///
/// The definition order of the systems can be different than their execution order if systems
/// are defined without constraint.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     Action, Built, DependsOn, EntityBuilder, EntityMainComponent, EntityRunner, system
/// # };
/// #
/// # fn system1() {}
/// # fn system2() {}
/// # fn system3() {}
/// # fn system4() {}
/// # fn system5() {}
/// #
/// struct MyEntity;
///
/// impl EntityMainComponent for MyEntity {
///     type Data = ();
///
///     fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder.with_self(Self)
///     }
///
///     fn on_update(runner: EntityRunner<'_, Self>) {
///         runner
///             // `system1` has no constraint
///             .run(system!(system1))
///             // `system2` will be run after `system3` because of `Action2::Constraint`
///             .run_as::<Action2>(system!(system2))
///             // `system3` has no constraint because of `Action1::Constraint`
///             .run_as::<Action1>(system!(system3))
///             // `system4` will be run after `system2` and `system3`
///             .run_constrained::<(DependsOn<Action1>, DependsOn<Action2>)>(system!(system4))
///             // `system5` will be run after `system4`
///             .and_then(system!(system5));
///     }
/// }
///
/// struct Action1 {}
///
/// impl Action for Action1 {
///     type Constraint = ();
/// }
///
/// struct Action2 {}
///
/// impl Action for Action2 {
///     type Constraint = DependsOn<Action1>;
/// }
/// ```
pub struct EntityRunner<'a, E> {
    core: &'a mut CoreStorage,
    phantom: PhantomData<E>,
}

impl<'a, E> EntityRunner<'a, E>
where
    E: EntityMainComponent,
{
    /// Adds a system to run during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    pub fn run(self, system: SystemBuilder) -> UsedEntityRunner<'a, E> {
        self.run_with_action(system, None, ActionDependencies::Types(vec![]))
    }

    /// Adds a system to run during each [`App`](crate::App) update that is associated to an action.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    ///
    /// The constraints of the system are defined by `<A as Action>::Constraint`.
    pub fn run_as<A>(self, system: SystemBuilder) -> UsedEntityRunner<'a, E>
    where
        A: Action,
    {
        self.run_with_action(
            system,
            Some(TypeId::of::<A>()),
            ActionDependencies::Types(A::Constraint::dependency_types()),
        )
    }

    /// Adds a system with constraints to run during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    ///
    /// The constraints of the system are defined by `C`.
    pub fn run_constrained<C>(self, system: SystemBuilder) -> UsedEntityRunner<'a, E>
    where
        C: ActionConstraint,
    {
        self.run_with_action(
            system,
            None,
            ActionDependencies::Types(C::dependency_types()),
        )
    }

    fn run_with_action(
        self,
        system: SystemBuilder,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> UsedEntityRunner<'a, E> {
        let properties = (system.properties_fn)(self.core);
        UsedEntityRunner {
            latest_action_idx: self.core.add_system(
                system.wrapper,
                TypeId::of::<E>(),
                properties,
                action_type,
                action_dependencies,
            ),
            runner: self,
        }
    }
}

/// A type for defining the other systems of an entity.
///
/// Cyclic dependencies between systems are detected at compile time.
///
/// The definition order of the systems can be different than their execution order if systems
/// are defined without constraint.
///
/// # Examples
///
/// See [`EntityRunner`](crate::EntityRunner).
pub struct UsedEntityRunner<'a, E> {
    runner: EntityRunner<'a, E>,
    latest_action_idx: ActionIdx,
}

impl<'a, E> UsedEntityRunner<'a, E>
where
    E: EntityMainComponent,
{
    /// Adds a system to run during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    pub fn run(self, system: SystemBuilder) -> Self {
        self.runner.run(system)
    }

    /// Adds a system to run during each [`App`](crate::App) update that is associated to an action.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    ///
    /// The constraints of the system are defined by `<A as Action>::Constraint`.
    pub fn run_as<A>(self, system: SystemBuilder) -> Self
    where
        A: Action,
    {
        self.runner.run_as::<A>(system)
    }

    /// Adds a system with constraints to run during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    ///
    /// The constraints of the system are defined by `C`.
    pub fn run_constrained<C>(self, system: SystemBuilder) -> Self
    where
        C: ActionConstraint,
    {
        self.runner.run_constrained::<C>(system)
    }

    /// Adds a system to run after the previous defined one during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    pub fn and_then(self, system: SystemBuilder) -> Self {
        self.runner.run_with_action(
            system,
            None,
            ActionDependencies::Action(self.latest_action_idx),
        )
    }
}

mod internal {
    use crate::storages::archetypes::EntityLocation;
    use crate::storages::components::ComponentTypeIdx;
    use crate::storages::core::CoreStorage;
    use std::any::Any;

    pub struct StorageWrapper<'a>(pub(super) &'a mut CoreStorage);

    pub trait ComponentAdd {
        fn add(self, storage: &mut StorageWrapper<'_>, location: EntityLocation);
    }

    pub struct AddedComponents<C, A> {
        pub(super) component: C,
        pub(super) type_idx: ComponentTypeIdx,
        pub(super) is_added: bool,
        pub(super) other_components: A,
    }

    impl<C, A> ComponentAdd for AddedComponents<C, A>
    where
        C: Any + Sync + Send,
        A: ComponentAdd,
    {
        fn add(self, storage: &mut StorageWrapper<'_>, location: EntityLocation) {
            self.other_components.add(storage, location);
            if self.is_added {
                storage
                    .0
                    .add_component(self.component, self.type_idx, location);
            }
        }
    }

    impl ComponentAdd for () {
        fn add(self, _storage: &mut StorageWrapper<'_>, _location: EntityLocation) {}
    }
}

#[cfg(test)]
mod built_tests {
    use crate::Built;
    use std::panic::{RefUnwindSafe, UnwindSafe};

    assert_impl_all!(Built: Sync, Send, Unpin, UnwindSafe, RefUnwindSafe);
}

#[cfg(test)]
mod entity_builder_tests {
    use crate::storages::archetypes::{ArchetypeEntityPos, ArchetypeIdx, ArchetypeStorage};
    use crate::storages::core::CoreStorage;
    use crate::{Built, EntityBuilder, EntityMainComponent};
    use std::any::Any;
    use std::fmt::Debug;
    use std::marker::PhantomData;

    assert_impl_all!(EntityBuilder<'_, ParentEntity>: Send, Unpin);

    #[derive(Debug, PartialEq)]
    struct ParentEntity(u32);

    impl EntityMainComponent for ParentEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[derive(Debug, PartialEq)]
    struct ChildEntity(u32);

    impl EntityMainComponent for ChildEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn build_entity() {
        let mut core = CoreStorage::default();
        let builder = EntityBuilder::<ChildEntity> {
            core: &mut core,
            src_location: None,
            dst_archetype_idx: ArchetypeStorage::DEFAULT_IDX,
            added_components: (),
            phantom: PhantomData,
        };
        builder
            .with(10_u32)
            .with_if(0_i64, true)
            .with_if(20_i64, true)
            .with_if(30_i8, false)
            .inherit_from::<ParentEntity>(40)
            .with_self(ChildEntity(50));
        let archetype_idx = ArchetypeIdx::from(5);
        let archetype_pos = ArchetypeEntityPos::from(0);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&10_u32));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&20_i64));
        assert_component_eq::<i8>(&core, archetype_idx, archetype_pos, None);
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&ParentEntity(40)));
        assert_component_eq(&core, archetype_idx, archetype_pos, Some(&ChildEntity(50)));
    }

    fn assert_component_eq<C>(
        core: &CoreStorage,
        archetype_idx: ArchetypeIdx,
        archetype_pos: ArchetypeEntityPos,
        expected_component: Option<&C>,
    ) where
        C: Any + PartialEq + Debug,
    {
        assert_eq!(
            core.components()
                .read_components::<C>()
                .get(archetype_idx)
                .and_then(|c| c.get(archetype_pos)),
            expected_component
        );
    }
}

#[cfg(test)]
mod entity_runner_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::SystemProperties;
    use crate::{
        Action, Built, DependsOn, EntityBuilder, EntityMainComponent, EntityRunner, SystemBuilder,
    };
    use std::marker::PhantomData;
    assert_impl_all!(EntityRunner<'_, TestEntity>: Send, Unpin);

    struct TestActionDependency;

    impl Action for TestActionDependency {
        type Constraint = ();
    }

    struct TestAction;

    impl Action for TestAction {
        type Constraint = DependsOn<TestActionDependency>;
    }

    struct TestEntity(u32);

    impl EntityMainComponent for TestEntity {
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn run_systems() {
        let mut core = CoreStorage::default();
        core.add_entity_type::<TestEntity>();
        let runner = EntityRunner::<TestEntity> {
            core: &mut core,
            phantom: PhantomData,
        };
        runner
            .run(create_system_builder())
            .run(create_system_builder())
            .and_then(create_system_builder())
            .run_as::<TestAction>(create_system_builder())
            .run_as::<TestActionDependency>(create_system_builder())
            .run_constrained::<DependsOn<TestActionDependency>>(create_system_builder());
        let actions = &core.system_data().actions;
        assert_eq!(actions.system_counts(), ti_vec![1; 6]);
        assert_eq!(actions.dependency_idxs(0.into()), []);
        assert_eq!(actions.dependency_idxs(1.into()), []);
        assert_eq!(actions.dependency_idxs(2.into()), [1.into()]);
        assert_eq!(actions.dependency_idxs(3.into()), []);
        assert_eq!(actions.dependency_idxs(4.into()), [3.into()]);
        assert_eq!(actions.dependency_idxs(5.into()), [3.into()]);
    }

    fn create_system_builder() -> SystemBuilder {
        SystemBuilder {
            properties_fn: |_| SystemProperties {
                component_types: vec![],
                can_update: false,
                archetype_filter: ArchetypeFilter::None,
            },
            wrapper: |_, _| (),
        }
    }
}
