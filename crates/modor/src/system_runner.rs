use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::{Action, ActionConstraint, SystemBuilder};
use std::any::TypeId;

/// A type for defining systems to run during each [`App`](crate::App) update.
///
/// Cyclic dependencies between systems are detected at compile time.<br>
/// The definition order of the systems can be different than their execution order if systems
/// are defined without constraint.
///
/// See [`system!`](crate::system!) for more information about systems.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     Built, DependsOn, EntityBuilder, EntityMainComponent, SystemRunner, system, define_action
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
/// impl MyEntity {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///     }
/// }
///
/// impl EntityMainComponent for MyEntity {
///     type Type = ();
///
///     fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
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
///             .and_then(system!(system5))
///     }
/// }
///
/// define_action!(Action1);
/// define_action!(Action2: Action1);
/// ```
pub struct SystemRunner<'a> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) entity_type_idx: ComponentTypeIdx,
    pub(crate) latest_action_idx: Option<ActionIdx>,
}

impl<'a> SystemRunner<'a> {
    /// Adds a system.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    pub fn run(self, system: SystemBuilder) -> SystemRunner<'a> {
        self.run_with_action(system, None, ActionDependencies::Types(vec![]))
    }

    /// Adds a system associated to an action.
    ///
    /// The constraints of the system are defined by `<A as Action>::Constraint`.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    pub fn run_as<A>(self, system: SystemBuilder) -> SystemRunner<'a>
    where
        A: Action,
    {
        self.run_with_action(
            system,
            Some(TypeId::of::<A>()),
            ActionDependencies::Types(A::Constraint::dependency_types()),
        )
    }

    /// Adds a system with constraints `C`.
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    pub fn run_constrained<C>(self, system: SystemBuilder) -> SystemRunner<'a>
    where
        C: ActionConstraint,
    {
        self.run_with_action(
            system,
            None,
            ActionDependencies::Types(C::dependency_types()),
        )
    }

    /// Adds a system to run after the previous defined one.
    ///
    /// The added system does not have an associated action, and the action of the previous
    /// defined system is not inherited.<br>
    /// If no system has been previously defined, this method has the same effect as
    /// [`SystemRunner::run`](crate::SystemRunner::run).
    ///
    /// The [`system!`](crate::system!) macro must be used to define the `system`.
    pub fn and_then(self, system: SystemBuilder) -> SystemRunner<'a> {
        if let Some(latest_action_idx) = self.latest_action_idx {
            self.run_with_action(system, None, ActionDependencies::Action(latest_action_idx))
        } else {
            self.run(system)
        }
    }

    fn run_with_action(
        self,
        system: SystemBuilder,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> SystemRunner<'a> {
        let mut properties = (system.properties_fn)(self.core);
        properties
            .filtered_component_type_idxs
            .push(self.entity_type_idx);
        SystemRunner {
            latest_action_idx: Some(self.core.add_system(
                system.wrapper,
                properties,
                action_type,
                action_dependencies,
            )),
            core: self.core,
            entity_type_idx: self.entity_type_idx,
        }
    }
}

#[cfg(test)]
mod entity_runner_tests {
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::SystemProperties;
    use crate::{Action, DependsOn, SystemBuilder, SystemRunner};

    struct TestActionDependency;

    impl Action for TestActionDependency {
        type Constraint = ();
    }

    struct TestAction;

    impl Action for TestAction {
        type Constraint = DependsOn<TestActionDependency>;
    }

    create_entity_type!(TestEntity);

    assert_impl_all!(SystemRunner<'_>: Send, Unpin);

    #[test]
    fn run_systems() {
        let mut core = CoreStorage::default();
        let entity_type_idx = core.add_entity_type::<TestEntity>();
        let runner = SystemRunner {
            core: &mut core,
            entity_type_idx,
            latest_action_idx: None,
        };
        runner
            .and_then(create_system_builder())
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
        let filtered_type_idxs = core.systems().filtered_component_idxs(0.into());
        assert_eq!(filtered_type_idxs, [entity_type_idx]);
    }

    fn create_system_builder() -> SystemBuilder {
        SystemBuilder {
            properties_fn: |_| SystemProperties {
                component_types: vec![],
                can_update: false,
                filtered_component_type_idxs: vec![],
            },
            wrapper: |_, _| (),
        }
    }
}
