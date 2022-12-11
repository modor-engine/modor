use super::archetypes::ArchetypeIdx;
use super::components::ComponentTypeIdx;
use super::systems::SystemIdx;
use modor_internal::ti_vec::TiVecSafeOperations;
use typed_index_collections::TiVec;

#[derive(Default)]
pub(crate) struct ArchetypeStateStorage {
    are_systems_new: TiVec<SystemIdx, bool>,
    mutations: TiVec<SystemIdx, TiVec<ComponentTypeIdx, Option<TiVec<ArchetypeIdx, bool>>>>,
}

impl ArchetypeStateStorage {
    pub(crate) fn is_mutated(
        &self,
        system_idx: SystemIdx,
        component_type_idx: ComponentTypeIdx,
        archetype_idx: ArchetypeIdx,
    ) -> bool {
        self.are_systems_new[system_idx]
            || self.mutations[system_idx][component_type_idx]
                .as_ref()
                .expect("internal error: component not tracked for mutability")
                .get(archetype_idx)
                .copied()
                .unwrap_or(false)
    }

    pub(super) fn add_system(
        &mut self,
        system_idx: SystemIdx,
        mutation_component_type_idxs: &[ComponentTypeIdx],
    ) {
        let mutations = self.mutations.get_mut_or_create(system_idx);
        for &component_type_idx in mutation_component_type_idxs {
            *mutations.get_mut_or_create(component_type_idx) = Some(ti_vec![]);
        }
        *self.are_systems_new.get_mut_or_create(system_idx) = true;
    }

    pub(super) fn reset_system(&mut self, system_idx: SystemIdx) {
        for state in self.mutations[system_idx].iter_mut().flatten().flatten() {
            *state = false;
        }
    }

    pub(crate) fn add_mutated_component(
        &mut self,
        component_type_idx: ComponentTypeIdx,
        archetype_idx: ArchetypeIdx,
        excluded_system: Option<SystemIdx>,
    ) {
        for (system_idx, system_mutations) in self.mutations.iter_mut_enumerated() {
            if let Some(Some(archetypes)) = system_mutations.get_mut(component_type_idx) {
                if excluded_system != Some(system_idx) {
                    *archetypes.get_mut_or_create(archetype_idx) = true;
                }
            }
        }
    }

    pub(super) fn reset_state(&mut self) {
        for is_new in &mut self.are_systems_new {
            *is_new = false;
        }
    }
}
