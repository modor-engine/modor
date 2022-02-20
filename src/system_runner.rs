use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::core::{CoreStorage, SystemCallerType};
use crate::system_runner::internal::{FirstSystem, OtherSystems};
use crate::{Action, ActionConstraint, SystemBuilder};
use std::any::TypeId;
use std::marker::PhantomData;

/// A type for defining system to run during update.
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
/// #     Action, Built, DependsOn, EntityBuilder, EntityMainComponent, SystemRunner, system
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
///     fn on_update(runner: SystemRunner<'_>) {
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
pub struct SystemRunner<'a, T = FirstSystem> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) caller_type: SystemCallerType,
    pub(crate) latest_action_idx: Option<ActionIdx>,
    pub(crate) phantom: PhantomData<T>,
}

impl<'a, T> SystemRunner<'a, T> {
    /// Adds a system to run during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    pub fn run(self, system: SystemBuilder) -> SystemRunner<'a, OtherSystems> {
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
    pub fn run_as<A>(self, system: SystemBuilder) -> SystemRunner<'a, OtherSystems>
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
    pub fn run_constrained<C>(self, system: SystemBuilder) -> SystemRunner<'a, OtherSystems>
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
    ) -> SystemRunner<'a, OtherSystems> {
        let properties = (system.properties_fn)(self.core);
        SystemRunner {
            latest_action_idx: Some(self.core.add_system(
                system.wrapper,
                self.caller_type,
                properties,
                action_type,
                action_dependencies,
            )),
            core: self.core,
            caller_type: self.caller_type,
            phantom: PhantomData,
        }
    }
}

impl<'a> SystemRunner<'a, OtherSystems> {
    /// Adds a system to run after the previous defined one during each [`App`](crate::App) update.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    ///
    /// If the system is iterative (see [`system!`](crate::system!) for more information),
    /// the system iterates only on entities containing a component of type `E`.
    pub fn and_then(self, system: SystemBuilder) -> SystemRunner<'a, OtherSystems> {
        let latest_action_idx = self
            .latest_action_idx
            .expect("internal error: no previous system defined");
        self.run_with_action(system, None, ActionDependencies::Action(latest_action_idx))
    }
}

mod internal {
    pub struct FirstSystem;

    pub struct OtherSystems;
}

#[cfg(test)]
mod entity_runner_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::{CoreStorage, SystemCallerType};
    use crate::storages::systems::SystemProperties;
    use crate::{
        Action, Built, DependsOn, EntityBuilder, EntityMainComponent, SystemBuilder, SystemRunner,
    };
    use std::any::TypeId;
    use std::marker::PhantomData;

    assert_impl_all!(SystemRunner<'_>: Send, Unpin);

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
        let runner: SystemRunner<'_> = SystemRunner {
            core: &mut core,
            caller_type: SystemCallerType::Entity(TypeId::of::<TestEntity>()),
            latest_action_idx: None,
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
                globals: vec![],
                can_update: false,
                archetype_filter: ArchetypeFilter::None,
            },
            wrapper: |_, _| (),
        }
    }
}
