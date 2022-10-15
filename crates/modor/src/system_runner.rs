use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::systems::internal::SystemWrapper;
use crate::{Action, ActionConstraint, SystemBuilder};
use std::any::TypeId;

#[doc(hidden)]
pub struct SystemRunner<'a> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) entity_type_idx: ComponentTypeIdx,
    pub(crate) latest_action_idx: Option<ActionIdx>,
}

#[allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]
impl<'a> SystemRunner<'a> {
    #[doc(hidden)]
    pub fn run(self, system: SystemBuilder<SystemWrapper>) -> Self {
        self.run_with_action(system, None, ActionDependencies::Types(vec![]))
    }

    #[doc(hidden)]
    pub fn run_as<A>(self, system: SystemBuilder<SystemWrapper>) -> Self
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
    pub fn run_constrained<C>(self, system: SystemBuilder<SystemWrapper>) -> Self
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
    pub fn and_then(self, system: SystemBuilder<SystemWrapper>) -> Self {
        if let Some(latest_action_idx) = self.latest_action_idx {
            self.run_with_action(system, None, ActionDependencies::Action(latest_action_idx))
        } else {
            self.run(system)
        }
    }

    fn run_with_action(
        self,
        system: SystemBuilder<SystemWrapper>,
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
