use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::{Action, ActionConstraint, SystemBuilder};
use std::any::TypeId;

#[doc(hidden)]
pub struct SystemRunner<'a> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) entity_type_idx: ComponentTypeIdx,
    pub(crate) latest_action_idx: Option<ActionIdx>,
}

#[allow(clippy::return_self_not_must_use)]
impl<'a> SystemRunner<'a> {
    #[doc(hidden)]
    pub fn run(self, system: SystemBuilder) -> Self {
        self.run_with_action(system, None, ActionDependencies::Types(vec![]))
    }

    #[doc(hidden)]
    pub fn run_as<A>(self, system: SystemBuilder) -> Self
    where
        A: Action,
    {
        self.run_with_action(
            system,
            Some(TypeId::of::<A>()),
            ActionDependencies::Types(A::Constraint::dependency_types()),
        )
    }

    #[doc(hidden)]
    pub fn run_constrained<C>(self, system: SystemBuilder) -> Self
    where
        C: ActionConstraint,
    {
        self.run_with_action(
            system,
            None,
            ActionDependencies::Types(C::dependency_types()),
        )
    }

    #[doc(hidden)]
    pub fn and_then(self, system: SystemBuilder) -> Self {
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
