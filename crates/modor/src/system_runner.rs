use crate::storages::actions::{ActionDependencies, ActionIdx};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::FullSystemProperties;
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
    pub fn run(self, system: SystemBuilder<SystemWrapper>, label: &'static str) -> Self {
        self.run_with_action(system, label, None, ActionDependencies::Types(vec![]))
    }

    #[doc(hidden)]
    pub fn run_as<A>(self, system: SystemBuilder<SystemWrapper>, label: &'static str) -> Self
    where
        A: Action,
    {
        self.run_with_action(
            system,
            label,
            Some(TypeId::of::<A>()),
            ActionDependencies::Types(A::Constraint::dependency_types()),
        )
    }

    #[doc(hidden)]
    pub fn run_constrained<C>(
        self,
        system: SystemBuilder<SystemWrapper>,
        label: &'static str,
    ) -> Self
    where
        C: ActionConstraint,
    {
        self.run_with_action(
            system,
            label,
            None,
            ActionDependencies::Types(C::dependency_types()),
        )
    }

    #[doc(hidden)]
    pub fn and_then(self, system: SystemBuilder<SystemWrapper>, label: &'static str) -> Self {
        if let Some(latest_action_idx) = self.latest_action_idx {
            self.run_with_action(
                system,
                label,
                None,
                ActionDependencies::Action(latest_action_idx),
            )
        } else {
            self.run(system, label)
        }
    }

    fn run_with_action(
        self,
        system: SystemBuilder<SystemWrapper>,
        label: &'static str,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> SystemRunner<'a> {
        let properties = (system.properties_fn)(self.core);
        SystemRunner {
            latest_action_idx: Some(self.core.add_system(
                system.wrapper,
                label,
                FullSystemProperties {
                    component_types: properties.component_types,
                    can_update: properties.can_update,
                    archetype_filter_fn: system.archetype_filter_fn,
                    entity_type: Some(self.entity_type_idx),
                },
                action_type,
                action_dependencies,
            )),
            core: self.core,
            entity_type_idx: self.entity_type_idx,
        }
    }
}
