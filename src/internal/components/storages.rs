use crate::internal::components::interfaces::Components;
use fxhash::FxHashMap;
use std::any::{Any, TypeId};
use std::sync::RwLock;

type CreateArchetypeFn = fn(&mut ComponentStorage, usize) -> usize;
type DeleteArchetypeFn = fn(&mut ComponentStorage, usize, usize);
type MoveFn = fn(&mut ComponentStorage, usize, usize, usize, usize);
type DeleteFn = fn(&mut ComponentStorage, usize, usize, usize);

#[derive(Default)]
pub(super) struct ComponentStorage {
    components: Vec<Box<dyn Any + Sync + Send>>,
    create_archetype_fns: Vec<CreateArchetypeFn>,
    delete_archetype_fns: Vec<DeleteArchetypeFn>,
    move_fns: Vec<MoveFn>,
    delete_fns: Vec<DeleteFn>,
}

impl ComponentStorage {
    pub(super) fn create_type<C>(&mut self)
    where
        C: Any + Sync + Send,
    {
        self.components.push(Box::new(Vec::<Vec<C>>::new()));
        let create_archetype_fn = Self::create_archetype_generic::<C>;
        self.create_archetype_fns.push(create_archetype_fn);
        let move_fn = Self::move_generic::<C>;
        self.move_fns.push(move_fn);
        let delete_fn = Self::delete_generic::<C>;
        self.delete_fns.push(delete_fn);
        let delete_archetype_fn = Self::delete_archetype_generic::<C>;
        self.delete_archetype_fns.push(delete_archetype_fn);
    }

    pub(super) fn delete_archetype(&mut self, type_idx: usize, archetype_pos: usize) {
        let delete_archetype_fn = self.delete_archetype_fns[type_idx];
        delete_archetype_fn(self, type_idx, archetype_pos)
    }

    pub(super) fn exists<C>(&self, type_idx: usize, archetype_pos: usize, entity_pos: usize) -> bool
    where
        C: Any,
    {
        let components = self.downcast_components::<C>(type_idx);
        components[archetype_pos].get(entity_pos).is_some()
    }

    pub(super) fn export(&mut self) -> Vec<RwLock<Components>> {
        self.components
            .drain(..)
            .map(Components::from)
            .map(RwLock::from)
            .collect::<Vec<_>>()
    }

    /// `components` must be the object returned previously by the `export` method.
    /// `components` structure must not have been modified between the `export` and the `import`.
    /// The storage must also not have been used between the `export` and the `import`.
    pub(super) fn import(&mut self, components: &mut Vec<RwLock<Components>>) {
        self.components.extend(
            components
                .drain(..)
                .map(RwLock::into_inner)
                .map(Result::unwrap)
                .map(Components::into),
        );
    }

    pub(super) fn add<C>(&mut self, type_idx: usize, archetype_pos: usize, component: C)
    where
        C: Any,
    {
        let components = self.downcast_components_mut(type_idx);
        (components.len()..=archetype_pos).for_each(|_| components.push(Vec::new()));
        components[archetype_pos].push(component);
    }

    pub(super) fn replace<C>(
        &mut self,
        type_idx: usize,
        archetype_pos: usize,
        entity_pos: usize,
        component: C,
    ) where
        C: Any,
    {
        let components = self.downcast_components_mut(type_idx);
        components[archetype_pos][entity_pos] = component;
    }

    pub(super) fn move_(
        &mut self,
        type_idx: usize,
        src_archetype_pos: usize,
        src_entity_pos: usize,
        dst_archetype_pos: usize,
    ) {
        let move_fn = self.move_fns[type_idx];
        move_fn(
            self,
            type_idx,
            src_archetype_pos,
            src_entity_pos,
            dst_archetype_pos,
        )
    }

    pub(super) fn delete(&mut self, type_idx: usize, archetype_pos: usize, entity_pos: usize) {
        let swap_delete_fn = self.delete_fns[type_idx];
        swap_delete_fn(self, type_idx, archetype_pos, entity_pos)
    }

    fn create_archetype_generic<C>(&mut self, type_idx: usize) -> usize
    where
        C: Any,
    {
        let components = self.downcast_components_mut::<C>(type_idx);
        let archetype_idx = components.len();
        components.push(Vec::new());
        archetype_idx
    }

    fn move_generic<C>(
        &mut self,
        type_idx: usize,
        src_archetype_pos: usize,
        src_entity_pos: usize,
        dst_archetype_pos: usize,
    ) where
        C: Any,
    {
        let components = self.downcast_components_mut::<C>(type_idx);
        (components.len()..=dst_archetype_pos).for_each(|_| components.push(Vec::new()));
        let component = components[src_archetype_pos].swap_remove(src_entity_pos);
        components[dst_archetype_pos].push(component);
    }

    fn delete_generic<C>(&mut self, type_idx: usize, archetype_pos: usize, entity_pos: usize)
    where
        C: Any,
    {
        let components = self.downcast_components_mut::<C>(type_idx);
        components[archetype_pos].swap_remove(entity_pos);
    }

    fn delete_archetype_generic<C>(&mut self, type_idx: usize, archetype_pos: usize)
    where
        C: Any,
    {
        let components = self.downcast_components_mut::<C>(type_idx);
        if let Some(archetype_pos) = components.get_mut(archetype_pos) {
            *archetype_pos = Vec::new();
        }
    }

    fn downcast_components<C>(&self, type_idx: usize) -> &Vec<Vec<C>>
    where
        C: Any,
    {
        let type_components = &self.components[type_idx];
        type_components
            .downcast_ref()
            .expect("internal error: downcast component storage with wrong component type")
    }

    fn downcast_components_mut<C>(&mut self, type_idx: usize) -> &mut Vec<Vec<C>>
    where
        C: Any,
    {
        let type_components = &mut self.components[type_idx];
        type_components
            .downcast_mut()
            .expect("internal error: mutably downcast component storage with wrong component type")
    }
}

#[derive(Default)]
pub(super) struct ArchetypePositionStorage {
    positions: Vec<Vec<Option<usize>>>,
    deleted_positions: Vec<Vec<usize>>,
    next_positions: Vec<usize>,
}

impl ArchetypePositionStorage {
    pub(super) fn get(&self, type_idx: usize, archetype_idx: usize) -> Option<usize> {
        let type_positions = &self.positions[type_idx];
        type_positions.get(archetype_idx).copied()?
    }

    pub(super) fn create_type(&mut self) {
        self.positions.push(Vec::new());
        self.deleted_positions.push(Vec::new());
        self.next_positions.push(0);
    }

    pub(super) fn create(&mut self, type_idx: usize, archetype_idx: usize) -> usize {
        let archetype_pos = self.deleted_positions[type_idx].pop().unwrap_or_else(|| {
            let pos = self.next_positions[type_idx];
            self.next_positions[type_idx] += 1;
            pos
        });
        let type_positions = &mut self.positions[type_idx];
        (type_positions.len()..=archetype_idx).for_each(|_| type_positions.push(None));
        type_positions[archetype_idx] = Some(archetype_pos);
        archetype_pos
    }

    pub(super) fn delete(&mut self, type_idx: usize, archetype_idx: usize) {
        let type_positions = &mut self.positions[type_idx];
        if let Some(archetype_pos) = type_positions[archetype_idx].take() {
            self.deleted_positions[type_idx].push(archetype_pos);
        }
    }
}

#[derive(Default)]
pub(super) struct TypeStorage(FxHashMap<TypeId, usize>);

impl TypeStorage {
    pub(super) fn idx(&self, type_id: TypeId) -> Option<usize> {
        self.0.get(&type_id).copied()
    }

    pub(super) fn add(&mut self, type_id: TypeId) {
        let idx = self.0.len();
        self.0.insert(type_id, idx);
    }
}

#[cfg(test)]
mod component_storage_tests {
    use super::*;
    use std::fmt::Debug;

    fn assert_eq_components<C>(
        components: &[RwLock<Components>],
        type_idx: usize,
        expected: &[Vec<C>],
    ) where
        C: Any + Debug + PartialEq,
    {
        assert!(components.len() > type_idx);
        let guard = &components[type_idx].try_read().unwrap();
        let downcast_components = guard.0.downcast_ref::<Vec<Vec<C>>>().unwrap();
        assert_eq!(downcast_components, expected);
    }

    #[test]
    fn default() {
        let mut storage = ComponentStorage::default();

        let components = storage.export();
        assert!(components.is_empty());
    }

    #[test]
    fn create_types() {
        let mut storage = ComponentStorage::default();

        storage.create_type::<u32>();
        storage.create_type::<i64>();

        let components = storage.export();
        assert_eq!(components.len(), 2);
        assert_eq_components::<u32>(&components, 0, &[]);
        assert_eq_components::<i64>(&components, 1, &[]);
    }

    #[test]
    #[should_panic]
    fn add_component_for_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.add::<u32>(0, 0, 0);
    }

    #[test]
    fn add_component_for_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<i64>();
        storage.create_type::<u32>();

        storage.add::<u32>(1, 2, 13);

        let components = storage.export();
        assert_eq_components::<i64>(&components, 0, &[]);
        assert_eq_components::<u32>(&components, 1, &[Vec::new(), Vec::new(), vec![13]]);
    }

    #[test]
    #[should_panic]
    fn add_component_with_wrong_type_for_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(1, 0, 13);

        storage.add(1, 0, "");
    }

    #[test]
    fn add_component_with_correct_type_for_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.create_type::<i64>();
        storage.add::<i64>(1, 0, 10);
        storage.add::<i64>(1, 1, 20);
        storage.add::<i64>(1, 2, 30);

        storage.add::<i64>(1, 2, 13);
        storage.add::<i64>(1, 2, 42);

        let components = storage.export();
        assert_eq_components::<u32>(&components, 0, &[]);
        assert_eq_components::<i64>(&components, 1, &[vec![10], vec![20], vec![30, 13, 42]]);
    }

    #[test]
    #[should_panic]
    fn replace_component_for_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.replace::<u32>(0, 0, 0, 0);
    }

    #[test]
    #[should_panic]
    fn replace_component_for_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.replace::<u32>(0, 0, 0, 0);
    }

    #[test]
    #[should_panic]
    fn replace_component_for_missing_entity() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 0, 10);

        storage.replace::<u32>(0, 0, 1, 0);
    }

    #[test]
    #[should_panic]
    fn replace_component_with_wrong_type_for_existing_entity() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 0, 10);

        storage.replace::<i64>(0, 0, 0, 42);
    }

    #[test]
    fn replace_component_with_correct_type_for_existing_entity() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<String>();
        storage.create_type::<u32>();
        storage.create_type::<i64>();
        storage.add::<i64>(2, 1, 10);
        storage.add::<i64>(2, 1, 20);
        storage.add::<i64>(2, 1, 30);
        storage.add::<i64>(2, 1, 40);

        storage.replace::<i64>(2, 1, 3, 200);

        let components = storage.export();
        assert_eq_components::<i64>(&components, 2, &[Vec::new(), vec![10, 20, 30, 200]]);
    }

    #[test]
    #[should_panic]
    fn move_component_for_non_existing_type() {
        let mut storage = ComponentStorage::default();

        storage.move_(0, 0, 0, 1);
    }

    #[test]
    #[should_panic]
    fn move_component_from_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.move_(0, 0, 0, 1);
    }

    #[test]
    #[should_panic]
    fn move_missing_component_from_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 1, 10);

        storage.move_(0, 1, 2, 0);
    }

    #[test]
    #[should_panic]
    fn move_existing_component_to_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 1, 10);

        storage.move_(0, 1, 0, 2);

        let components = storage.export();
        assert_eq_components::<i64>(&components, 2, &[Vec::new(), Vec::new(), vec![10]]);
    }

    #[test]
    fn move_existing_component_to_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<String>();
        storage.create_type::<u32>();
        storage.create_type::<i64>();
        storage.add::<i64>(2, 1, 10);
        storage.add::<i64>(2, 1, 20);
        storage.add::<i64>(2, 1, 30);
        storage.add::<i64>(2, 1, 40);

        storage.move_(2, 1, 3, 0);

        let components = storage.export();
        assert_eq_components::<i64>(&components, 2, &[vec![40], vec![10, 20, 30]]);
    }

    #[test]
    #[should_panic]
    fn delete_component_for_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.delete(0, 0, 0);
    }

    #[test]
    #[should_panic]
    fn delete_component_for_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.delete(0, 0, 0);
    }

    #[test]
    fn delete_component_for_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<String>();
        storage.create_type::<u32>();
        storage.create_type::<i64>();
        storage.add::<i64>(2, 1, 10);
        storage.add::<i64>(2, 1, 20);
        storage.add::<i64>(2, 1, 30);

        storage.delete(2, 1, 0);

        let components = storage.export();
        assert_eq_components::<i64>(&components, 2, &[Vec::new(), vec![30, 20]]);
    }

    #[test]
    #[should_panic]
    fn delete_archetype_components_for_missing_type() {
        let mut storage = ComponentStorage::default();

        storage.delete_archetype(0, 0);
    }

    #[test]
    fn delete_archetype_components_for_missing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.delete_archetype(0, 1);

        let components = storage.export();
        assert_eq_components::<u32>(&components, 0, &[]);
    }

    #[test]
    fn delete_archetype_components_for_existing_archetype() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.create_type::<i64>();
        storage.add::<u32>(0, 0, 10);
        storage.add::<i64>(1, 1, 30);
        storage.add::<i64>(1, 2, 40);
        storage.add::<i64>(1, 2, 50);

        storage.delete_archetype(1, 2);

        let components = storage.export();
        assert_eq_components::<u32>(&components, 0, &[vec![10]]);
        assert_eq_components::<i64>(&components, 1, &[Vec::new(), vec![30], Vec::new()]);
    }

    #[test]
    fn export_components() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 0, 42);

        let components = storage.export();

        assert_eq!(components.len(), 1);
        assert_eq_components::<u32>(&components, 0, &[vec![42]]);
        assert!(storage.export().is_empty())
    }

    #[test]
    fn import_components() {
        let mut storage = ComponentStorage::default();
        let type1_components = Components::from(Box::new(vec![vec![10_u32], Vec::new()]) as Box<_>);
        let type2_components = Components::from(Box::new(vec![vec![20_i64, 30]]) as Box<_>);
        let mut components = vec![RwLock::new(type1_components), RwLock::new(type2_components)];

        storage.import(&mut components);

        assert!(components.is_empty());
        let components = storage.export();
        assert_eq!(components.len(), 2);
        assert_eq_components::<u32>(&components, 0, &[vec![10], Vec::new()]);
        assert_eq_components::<i64>(&components, 1, &[vec![20, 30]]);
    }

    #[test]
    #[should_panic]
    fn retrieve_whether_component_exists_using_missing_type_idx() {
        let storage = ComponentStorage::default();

        storage.exists::<u32>(0, 1, 2);
    }

    #[test]
    #[should_panic]
    fn retrieve_whether_component_exists_using_wrong_type() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.exists::<i64>(0, 1, 2);
    }

    #[test]
    #[should_panic]
    fn retrieve_whether_component_exists_using_missing_archetype_pos() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();

        storage.exists::<u32>(0, 1, 2);
    }

    #[test]
    fn retrieve_whether_missing_component_exists() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 1, 10);
        storage.add::<u32>(0, 1, 20);

        let exists = storage.exists::<u32>(0, 1, 2);

        assert!(!exists);
    }

    #[test]
    fn retrieve_whether_existing_component_exists() {
        let mut storage = ComponentStorage::default();
        storage.create_type::<u32>();
        storage.add::<u32>(0, 1, 10);
        storage.add::<u32>(0, 1, 20);
        storage.add::<u32>(0, 1, 30);

        let exists = storage.exists::<u32>(0, 1, 2);

        assert!(exists);
    }
}

#[cfg(test)]
mod archetype_position_storage_tests {
    use super::*;

    #[test]
    fn default() {
        let storage = ArchetypePositionStorage::default();

        assert_panics!(storage.get(0, 0));
    }

    #[test]
    fn create_types() {
        let mut storage = ArchetypePositionStorage::default();

        storage.create_type();
        storage.create_type();

        assert!(storage.get(0, 0).is_none());
        assert!(storage.get(1, 0).is_none());
        assert_panics!(storage.get(2, 0));
    }

    #[test]
    #[should_panic]
    fn create_position_for_missing_type() {
        let mut storage = ArchetypePositionStorage::default();

        storage.create(0, 0);
    }

    #[test]
    fn create_positions_for_existing_type_without_deleted_positions() {
        let mut storage = ArchetypePositionStorage::default();
        storage.create_type();
        storage.create_type();

        let archetype1_pos = storage.create(1, 0);
        let archetype2_pos = storage.create(1, 2);

        assert_eq!(archetype1_pos, 0);
        assert_eq!(archetype2_pos, 1);
        assert_eq!(storage.get(1, 0), Some(0));
        assert_eq!(storage.get(1, 2), Some(1));
    }

    #[test]
    #[should_panic]
    fn delete_position_with_missing_type() {
        let mut storage = ArchetypePositionStorage::default();

        storage.delete(1, 0);
    }

    #[test]
    #[should_panic]
    fn delete_position_with_missing_archetype() {
        let mut storage = ArchetypePositionStorage::default();
        storage.create_type();
        storage.create_type();

        storage.delete(1, 0);
    }

    #[test]
    fn delete_position_with_existing_archetype_with_position() {
        let mut storage = ArchetypePositionStorage::default();
        storage.create_type();
        storage.create_type();
        storage.create(1, 0);
        storage.create(1, 2);

        storage.delete(1, 0);

        assert_eq!(storage.get(1, 0), None);
        assert_eq!(storage.get(1, 2), Some(1));
        storage.create(1, 3);
        storage.create(1, 4);
        storage.create(1, 5);
        assert_eq!(storage.get(1, 3), Some(0));
        assert_eq!(storage.get(1, 4), Some(2));
        assert_eq!(storage.get(1, 5), Some(3));
    }

    #[test]
    fn delete_position_with_existing_archetype_without_position() {
        let mut storage = ArchetypePositionStorage::default();
        storage.create_type();
        storage.create_type();
        storage.create(1, 2);

        storage.delete(1, 0);

        assert_eq!(storage.get(1, 0), None);
        assert_eq!(storage.get(1, 2), Some(0));
        storage.create(1, 3);
        assert_eq!(storage.get(1, 3), Some(1));
    }
}

#[cfg(test)]
mod type_storage_tests {
    use super::*;

    #[test]
    fn default() {
        let storage = TypeStorage::default();

        assert_eq!(storage.idx(TypeId::of::<u32>()), None);
    }

    #[test]
    fn add_types() {
        let mut storage = TypeStorage::default();

        storage.add(TypeId::of::<u32>());
        storage.add(TypeId::of::<i64>());

        assert_eq!(storage.idx(TypeId::of::<u32>()), Some(0));
        assert_eq!(storage.idx(TypeId::of::<i64>()), Some(1));
        assert_eq!(storage.idx(TypeId::of::<String>()), None);
    }
}
