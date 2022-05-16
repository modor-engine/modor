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
