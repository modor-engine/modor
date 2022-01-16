use fxhash::FxHashMap;
use std::any::TypeId;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct ActionStorage {
    idxs: FxHashMap<TypeId, ActionIdx>,
    dependency_idxs: TiVec<ActionIdx, Vec<ActionIdx>>,
    system_counts: TiVec<ActionIdx, usize>,
}

impl ActionStorage {
    pub(crate) fn dependency_idxs(&self, action_idx: ActionIdx) -> &[ActionIdx] {
        &self.dependency_idxs[action_idx]
    }

    pub(crate) fn system_counts(&self) -> TiVec<ActionIdx, usize> {
        self.system_counts.clone()
    }

    pub(super) fn idx_or_create(&mut self, definition: ActionDefinition) -> ActionIdx {
        if let Some(action_type) = definition.type_ {
            if let Some(&action_idx) = self.idxs.get(&action_type) {
                if self.dependency_idxs[action_idx].is_empty() {
                    let dependency_idxs =
                        self.convert_dependencies_to_idxs(definition.dependency_types);
                    self.dependency_idxs[action_idx] = dependency_idxs;
                }
                action_idx
            } else {
                let action_idx = self.create(definition.dependency_types);
                self.idxs.insert(action_type, action_idx);
                action_idx
            }
        } else {
            self.create(definition.dependency_types)
        }
    }

    pub(super) fn add_system(&mut self, action_idx: ActionIdx) {
        self.system_counts[action_idx] += 1;
    }

    fn create(&mut self, dependencies: ActionDependencies) -> ActionIdx {
        let dependency_idxs = self.convert_dependencies_to_idxs(dependencies);
        self.dependency_idxs.push(dependency_idxs);
        self.system_counts.push_and_get_key(0)
    }

    fn convert_dependencies_to_idxs(&mut self, dependencies: ActionDependencies) -> Vec<ActionIdx> {
        match dependencies {
            ActionDependencies::Types(action_types) => action_types
                .into_iter()
                .map(|t| {
                    self.idx_or_create(ActionDefinition {
                        type_: Some(t),
                        dependency_types: ActionDependencies::Types(vec![]),
                    })
                })
                .collect(),
            ActionDependencies::Action(action_idx) => {
                vec![action_idx]
            }
        }
    }
}

idx_type!(pub(crate) ActionIdx);

#[derive(Clone)]
pub(crate) struct ActionDefinition {
    pub(crate) type_: Option<TypeId>,
    pub(crate) dependency_types: ActionDependencies,
}

#[derive(Clone)]
pub(crate) enum ActionDependencies {
    Types(Vec<TypeId>),
    Action(ActionIdx),
}

#[cfg(test)]
mod action_storage_tests {
    use crate::storages::actions::{ActionDefinition, ActionDependencies, ActionStorage};
    use std::any::TypeId;

    #[test]
    fn create_action_without_type_and_without_dependency() {
        let mut storage = ActionStorage::default();
        let definition = ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        };

        let action_idx = storage.idx_or_create(definition);

        assert_eq!(action_idx, 0.into());
        assert_eq!(storage.dependency_idxs(action_idx), &[]);
        assert_eq!(storage.system_counts(), ti_vec![0]);
    }

    #[test]
    fn create_action_without_type_and_with_dependency() {
        let mut storage = ActionStorage::default();
        let action1_idx = storage.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let definition = ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Action(action1_idx),
        };

        let action2_idx = storage.idx_or_create(definition);

        assert_eq!(action2_idx, 1.into());
        assert_eq!(storage.dependency_idxs(action1_idx), &[]);
        assert_eq!(storage.dependency_idxs(action2_idx), &[action1_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0, 0]);
    }

    #[test]
    fn create_action_with_new_type() {
        let mut storage = ActionStorage::default();
        let definition = ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Types(vec![]),
        };

        let action_idx = storage.idx_or_create(definition);

        assert_eq!(action_idx, 0.into());
        assert_eq!(storage.dependency_idxs(action_idx), &[]);
        assert_eq!(storage.system_counts(), ti_vec![0]);
    }

    #[test]
    fn create_action_with_existing_type_with_initial_dependencies() {
        let mut storage = ActionStorage::default();
        let action1_idx = storage.idx_or_create(ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Action(1.into()),
        });
        let definition = ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Action(2.into()),
        };

        let action2_idx = storage.idx_or_create(definition);

        assert_eq!(action2_idx, action1_idx);
        assert_eq!(storage.dependency_idxs(action2_idx), &[1.into()]);
        assert_eq!(storage.system_counts(), ti_vec![0]);
    }

    #[test]
    fn create_action_with_existing_type_without_dependency_update() {
        let mut storage = ActionStorage::default();
        let definition = ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Types(vec![]),
        };
        let action1_idx = storage.idx_or_create(definition.clone());

        let action2_idx = storage.idx_or_create(definition);

        assert_eq!(action2_idx, action1_idx);
        assert_eq!(storage.dependency_idxs(action2_idx), &[]);
        assert_eq!(storage.system_counts(), ti_vec![0]);
    }

    #[test]
    fn create_action_with_existing_type_with_dependency_update() {
        let mut storage = ActionStorage::default();
        let action1_idx = storage.idx_or_create(ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let definition = ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Action(action1_idx),
        };

        let action2_idx = storage.idx_or_create(definition);

        assert_eq!(action2_idx, action1_idx);
        assert_eq!(storage.dependency_idxs(action2_idx), &[action1_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0]);
    }

    #[test]
    fn create_action_with_type_dependency() {
        let mut storage = ActionStorage::default();
        let action1_idx = storage.idx_or_create(ActionDefinition {
            type_: Some(TypeId::of::<u32>()),
            dependency_types: ActionDependencies::Types(vec![]),
        });
        let definition = ActionDefinition {
            type_: Some(TypeId::of::<i64>()),
            dependency_types: ActionDependencies::Types(vec![TypeId::of::<u32>()]),
        };

        let action2_idx = storage.idx_or_create(definition);

        assert_eq!(action2_idx, 1.into());
        assert_eq!(storage.dependency_idxs(action2_idx), &[action1_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0, 0]);
    }

    #[test]
    fn add_systems_to_actions() {
        let mut storage = ActionStorage::default();
        let action_idx = storage.idx_or_create(ActionDefinition {
            type_: None,
            dependency_types: ActionDependencies::Types(vec![]),
        });

        storage.add_system(action_idx);
        storage.add_system(action_idx);
        storage.add_system(action_idx);

        assert_eq!(storage.system_counts(), ti_vec![3]);
    }
}
