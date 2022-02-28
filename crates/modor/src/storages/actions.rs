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

    pub(super) fn idx_or_create(
        &mut self,
        type_: Option<TypeId>,
        dependencies: ActionDependencies,
    ) -> ActionIdx {
        if let Some(action_type) = type_ {
            if let Some(&action_idx) = self.idxs.get(&action_type) {
                if self.dependency_idxs[action_idx].is_empty() {
                    let dependency_idxs = self.convert_dependencies_to_idxs(dependencies);
                    self.dependency_idxs[action_idx] = dependency_idxs;
                }
                action_idx
            } else {
                let action_idx = self.create(dependencies);
                self.idxs.insert(action_type, action_idx);
                action_idx
            }
        } else {
            self.create(dependencies)
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
                .map(|t| self.idx_or_create(Some(t), ActionDependencies::Types(vec![])))
                .collect(),
            ActionDependencies::Action(action_idx) => {
                vec![action_idx]
            }
        }
    }
}

idx_type!(pub(crate) ActionIdx);

#[derive(Clone)]
pub(crate) enum ActionDependencies {
    Types(Vec<TypeId>),
    Action(ActionIdx),
}

#[cfg(test)]
mod action_storage_tests {
    use crate::storages::actions::{ActionDependencies, ActionStorage};
    use std::any::TypeId;

    #[test]
    fn configure_untyped_dependent_actions() {
        let mut storage = ActionStorage::default();
        let no_dep = ActionDependencies::Types(vec![]);
        let first_idx = storage.idx_or_create(None, no_dep);
        let first_dep = ActionDependencies::Action(first_idx);
        let dependent_idx = storage.idx_or_create(None, first_dep);
        storage.add_system(dependent_idx);
        storage.add_system(dependent_idx);
        assert_eq!(first_idx, 0.into());
        assert_eq!(dependent_idx, 1.into());
        assert_eq!(storage.dependency_idxs(first_idx), &[]);
        assert_eq!(storage.dependency_idxs(dependent_idx), &[first_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0, 2]);
    }

    #[test]
    fn configure_typed_actions_with_dependency_update() {
        let mut storage = ActionStorage::default();
        let type_ = Some(TypeId::of::<u32>());
        let no_dep = ActionDependencies::Types(vec![]);
        let first_idx = storage.idx_or_create(None, no_dep.clone());
        let first_dep = ActionDependencies::Action(first_idx);
        let typed_idx = storage.idx_or_create(type_, no_dep);
        let updated_idx = storage.idx_or_create(type_, first_dep);
        storage.add_system(typed_idx);
        assert_eq!([typed_idx, updated_idx], [1.into(); 2]);
        assert_eq!(storage.dependency_idxs(typed_idx), &[first_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0, 1]);
    }

    #[test]
    fn configure_typed_actions_without_dependency_update() {
        let mut storage = ActionStorage::default();
        let type1 = Some(TypeId::of::<u32>());
        let type2 = Some(TypeId::of::<i64>());
        let no_dep = ActionDependencies::Types(vec![]);
        let first_idx = storage.idx_or_create(None, no_dep.clone());
        let first_dep = ActionDependencies::Action(first_idx);
        let second_idx = storage.idx_or_create(type1, no_dep);
        let second_dep = ActionDependencies::Types(vec![type1.unwrap()]);
        let typed_idx = storage.idx_or_create(type2, second_dep);
        let updated_idx = storage.idx_or_create(type2, first_dep);
        storage.add_system(typed_idx);
        assert_eq!([typed_idx, updated_idx], [2.into(); 2]);
        assert_eq!(storage.dependency_idxs(typed_idx), &[second_idx]);
        assert_eq!(storage.system_counts(), ti_vec![0, 0, 1]);
    }
}
