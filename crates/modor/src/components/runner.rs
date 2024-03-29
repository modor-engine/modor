use crate::storages::actions::{ActionDependency, ActionIdx};
use crate::storages::components::ComponentTypeIdx;
use crate::storages::core::CoreStorage;
use crate::storages::systems::{FullSystemProperties, SystemProperties};
use crate::{Action, Constraint, SystemBuilder, SystemWrapper};
use std::any::TypeId;
use std::iter;

#[doc(hidden)]
pub struct SystemRunner<'a> {
    pub(crate) core: &'a mut CoreStorage,
    pub(crate) component_action_type: TypeId,
    pub(crate) component_type_idx: ComponentTypeIdx,
    pub(crate) action_idxs: Vec<ActionIdx>,
}

impl<'a> SystemRunner<'a> {
    #[doc(hidden)]
    pub fn run(self, system: SystemBuilder<SystemWrapper>, label: &'static str) -> Self {
        self.run_with_action(system, label, None, vec![])
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
            A::dependency_types()
                .into_iter()
                .map(ActionDependency::Type)
                .collect(),
        )
    }

    #[doc(hidden)]
    pub fn run_constrained<C>(
        self,
        system: SystemBuilder<SystemWrapper>,
        label: &'static str,
    ) -> Self
    where
        C: Constraint,
    {
        self.run_with_action(
            system,
            label,
            None,
            C::action_types()
                .into_iter()
                .map(ActionDependency::Type)
                .collect(),
        )
    }

    #[doc(hidden)]
    pub fn and_then<C>(self, system: SystemBuilder<SystemWrapper>, label: &'static str) -> Self
    where
        C: Constraint,
    {
        if let Some(&latest_action_idx) = self.action_idxs.last() {
            self.run_with_action(
                system,
                label,
                None,
                C::action_types()
                    .into_iter()
                    .map(ActionDependency::Type)
                    .chain(iter::once(ActionDependency::Idx(latest_action_idx)))
                    .collect(),
            )
        } else {
            self.run(system, label)
        }
    }

    pub fn finish(self, label: &'static str) -> FinishedSystemRunner {
        let dependencies = self
            .action_idxs
            .iter()
            .copied()
            .map(ActionDependency::Idx)
            .collect();
        let component_type = self.component_action_type;
        self.run_with_action(
            SystemBuilder {
                properties_fn: |_| SystemProperties {
                    component_types: vec![],
                    can_update: false,
                    mutation_component_type_idxs: vec![],
                },
                archetype_filter_fn: |_, _, _| false,
                wrapper: |_| (),
            },
            label,
            Some(component_type),
            dependencies,
        );
        FinishedSystemRunner
    }

    fn run_with_action(
        self,
        system: SystemBuilder<SystemWrapper>,
        label: &'static str,
        action_type: Option<TypeId>,
        action_dependencies: Vec<ActionDependency>,
    ) -> SystemRunner<'a> {
        let properties = (system.properties_fn)(self.core);
        let mut action_idxs = self.action_idxs.clone();
        if let Some(action_idx) = self.core.add_system(
            FullSystemProperties {
                wrapper: system.wrapper,
                component_types: properties.component_types,
                can_update: properties.can_update,
                mutation_component_type_idxs: properties.mutation_component_type_idxs,
                archetype_filter_fn: system.archetype_filter_fn,
                component_type_idx: Some(self.component_type_idx),
                label,
            },
            action_type,
            action_dependencies,
        ) {
            action_idxs.push(action_idx);
        }
        SystemRunner {
            action_idxs,
            component_action_type: self.component_action_type,
            core: self.core,
            component_type_idx: self.component_type_idx,
        }
    }
}

#[doc(hidden)]
#[non_exhaustive]
pub struct FinishedSystemRunner;
