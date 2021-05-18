use crate::{SystemData, SystemInfo, SystemWrapper};
use std::any::TypeId;
use std::convert::TryInto;
use std::num::NonZeroUsize;

#[derive(Default)]
pub(super) struct SystemStorage(Vec<Vec<SystemWrapper>>);

impl SystemStorage {
    pub(super) fn group_count(&self) -> usize {
        self.0.len()
    }

    pub(super) fn system_count(&self, group_idx: usize) -> usize {
        self.0[group_idx].len()
    }

    pub(super) fn run(
        &self,
        group_idx: usize,
        system_idx: usize,
        data: &SystemData<'_>,
        filtered_component_types: Vec<TypeId>,
    ) {
        let group_filter = (group_idx != 0).then(|| group_idx.try_into().unwrap());
        let info = SystemInfo::new(filtered_component_types, group_filter);
        self.0[group_idx][system_idx](data, info);
    }

    pub(super) fn add(&mut self, group_idx: usize, system: SystemWrapper) -> usize {
        (self.0.len()..=group_idx).for_each(|_| self.0.push(Vec::new()));
        self.0[group_idx].push(system);
        self.0[group_idx].len() - 1
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if let Some(systems) = self.0.get_mut(group_idx.get()) {
            *systems = Vec::new();
        }
    }
}

#[derive(Default)]
pub(super) struct EntityTypeStorage(Vec<Vec<Option<TypeId>>>);

impl EntityTypeStorage {
    pub(super) fn get(&self, group_idx: usize, system_idx: usize) -> Option<TypeId> {
        self.0[group_idx][system_idx]
    }

    pub(super) fn set(&mut self, group_idx: usize, system_idx: usize, entity_type: Option<TypeId>) {
        (self.0.len()..=group_idx).for_each(|_| self.0.push(Vec::new()));
        (self.0[group_idx].len()..=system_idx).for_each(|_| self.0[group_idx].push(None));
        self.0[group_idx][system_idx] = entity_type;
    }

    pub(super) fn delete(&mut self, group_idx: NonZeroUsize) {
        if let Some(systems) = self.0.get_mut(group_idx.get()) {
            *systems = Vec::new();
        }
    }
}

#[cfg(test)]
mod tests_system_storage {
    use super::*;
    use crate::internal::actions::ActionFacade;
    use crate::internal::components::ComponentFacade;
    use crate::internal::core::CoreFacade;
    use std::sync::Mutex;

    fn use_data<F, G>(use_data_fn: F, use_deleted_groups_fn: G)
    where
        F: FnOnce(&SystemData<'_>),
        G: FnOnce(&[usize]),
    {
        let core = CoreFacade::default();
        let mut components = ComponentFacade::default();
        let component_interface = components.components();
        let actions = Mutex::new(ActionFacade::default());
        let data = SystemData::new(&core, &component_interface, &actions);
        use_data_fn(&data);
        use_deleted_groups_fn(
            &actions
                .try_lock()
                .unwrap()
                .reset()
                .deleted_group_idxs
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn default() {
        let storage = SystemStorage::default();

        assert_eq!(storage.group_count(), 0);
        assert_panics!(storage.system_count(0));
    }

    #[test]
    fn add_first_system_to_group() {
        let mut storage = SystemStorage::default();

        let system_idx = storage.add(1, |data, info| {
            assert_eq!(info.group_idx, Some(1.try_into().unwrap()));
            assert_eq!(info.filtered_component_types, vec![TypeId::of::<u32>()]);
            data.actions_mut()
                .delete_group(1.try_into().unwrap());
        });

        assert_eq!(system_idx, 0);
        assert_eq!(storage.group_count(), 2);
        assert_eq!(storage.system_count(0), 0);
        assert_eq!(storage.system_count(1), 1);
        assert_panics!(storage.system_count(2));
        use_data(
            |data| storage.run(1, system_idx, data, vec![TypeId::of::<u32>()]),
            |deleted_group_idxs| assert_eq!(deleted_group_idxs, &[1]),
        );
        use_data(
            |data| assert_panics!(storage.run(2, 0, data, vec![TypeId::of::<u32>()])),
            |_| (),
        );
        use_data(
            |data| assert_panics!(storage.run(1, system_idx + 1, data, vec![TypeId::of::<u32>()])),
            |_| (),
        );
    }

    #[test]
    fn add_other_system_to_group() {
        let mut storage = SystemStorage::default();
        storage.add(1, |_, _| ());

        let system_idx = storage.add(1, |data, info| {
            assert_eq!(info.group_idx, Some(1.try_into().unwrap()));
            assert_eq!(info.filtered_component_types, vec![TypeId::of::<u32>()]);
            data.actions_mut()
                .delete_group(1.try_into().unwrap());
        });

        assert_eq!(system_idx, 1);
        assert_eq!(storage.group_count(), 2);
        assert_eq!(storage.system_count(0), 0);
        assert_eq!(storage.system_count(1), 2);
        assert_panics!(storage.system_count(2));
        use_data(
            |data| storage.run(1, system_idx, data, vec![TypeId::of::<u32>()]),
            |deleted_group_idxs| assert_eq!(deleted_group_idxs, &[1]),
        );
    }

    #[test]
    fn add_system_to_global_group() {
        let mut storage = SystemStorage::default();

        let system_idx = storage.add(0, |data, info| {
            assert_eq!(info.group_idx, None);
            assert_eq!(info.filtered_component_types, vec![TypeId::of::<u32>()]);
            data.actions_mut()
                .delete_group(1.try_into().unwrap());
        });

        assert_eq!(system_idx, 0);
        assert_eq!(storage.group_count(), 1);
        assert_eq!(storage.system_count(0), 1);
        assert_panics!(storage.system_count(1));
        use_data(
            |data| storage.run(0, system_idx, data, vec![TypeId::of::<u32>()]),
            |deleted_group_idxs| assert_eq!(deleted_group_idxs, &[1]),
        );
    }

    #[test]
    fn delete_missing_group() {
        let mut storage = SystemStorage::default();
        let system_idx = storage.add(1, |data, _| {
            data.actions_mut()
                .delete_group(1.try_into().unwrap());
        });

        storage.delete(2.try_into().unwrap());

        use_data(
            |data| storage.run(1, system_idx, data, vec![TypeId::of::<u32>()]),
            |deleted_group_idxs| assert_eq!(deleted_group_idxs, &[1]),
        );
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = SystemStorage::default();
        let system1_idx = storage.add(1, |data, _| {
            data.actions_mut()
                .delete_group(1.try_into().unwrap());
        });
        let system2_idx = storage.add(2, |data, _| {
            data.actions_mut()
                .delete_group(2.try_into().unwrap());
        });

        storage.delete(1.try_into().unwrap());

        use_data(
            |data| assert_panics!(storage.run(1, system1_idx, data, vec![TypeId::of::<u32>()])),
            |_| (),
        );
        use_data(
            |data| storage.run(2, system2_idx, data, vec![TypeId::of::<u32>()]),
            |deleted_group_idxs| assert_eq!(deleted_group_idxs, &[2]),
        );
    }
}

#[cfg(test)]
mod tests_entity_type_storage {
    use super::*;

    #[test]
    fn set_system_entity_type() {
        let mut storage = EntityTypeStorage::default();

        storage.set(2, 3, Some(TypeId::of::<u32>()));

        assert_eq!(storage.get(2, 3), Some(TypeId::of::<u32>()));
        assert_eq!(storage.get(2, 2), None);
        assert_panics!(storage.get(1, 3));
        assert_panics!(storage.get(3, 0));
    }

    #[test]
    fn delete_missing_group() {
        let mut storage = EntityTypeStorage::default();
        storage.set(1, 2, Some(TypeId::of::<u32>()));

        storage.delete(2.try_into().unwrap());

        assert_eq!(storage.get(1, 2), Some(TypeId::of::<u32>()));
    }

    #[test]
    fn delete_existing_group() {
        let mut storage = EntityTypeStorage::default();
        storage.set(1, 2, Some(TypeId::of::<i64>()));
        storage.set(2, 3, Some(TypeId::of::<u32>()));

        storage.delete(1.try_into().unwrap());

        assert_eq!(storage.get(2, 3), Some(TypeId::of::<u32>()));
        assert_panics!(storage.get(1, 2));
    }
}
