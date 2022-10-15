use std::any::{Any, TypeId};
use std::mem;
use std::sync::Mutex;

use crate::storages::actions::{ActionDependencies, ActionIdx, ActionStorage};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::{EntityIdx, EntityStorage};
use crate::storages::systems::{SystemProperties, SystemStorage};
use crate::storages::updates::UpdateStorage;
use crate::systems::internal::SystemWrapper;
use crate::{SystemData, SystemInfo};

#[derive(Default)]
pub struct CoreStorage {
    archetypes: ArchetypeStorage,
    entities: EntityStorage,
    components: ComponentStorage,
    actions: ActionStorage,
    systems: SystemStorage,
    updates: Mutex<UpdateStorage>,
}

impl CoreStorage {
    pub(crate) fn archetypes(&self) -> &ArchetypeStorage {
        &self.archetypes
    }

    pub(crate) fn components(&self) -> &ComponentStorage {
        &self.components
    }

    pub(crate) fn systems(&self) -> &SystemStorage {
        &self.systems
    }

    pub(crate) fn set_thread_count(&mut self, count: u32) {
        self.systems.set_thread_count(count);
    }

    pub(crate) fn register_component_type<C>(&mut self) -> ComponentTypeIdx
    where
        C: Any + Sync + Send,
    {
        self.components.type_idx_or_create::<C>()
    }

    pub(crate) fn add_entity_type<C>(&mut self) -> ComponentTypeIdx
    where
        C: Any + Sync + Send,
    {
        self.components.add_entity_type::<C>()
    }

    pub(crate) fn add_component_type<C>(
        &mut self,
        src_archetype_idx: ArchetypeIdx,
    ) -> (ComponentTypeIdx, ArchetypeIdx)
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.components.type_idx_or_create::<C>();
        if let Ok(dst_archetype_idx) = self.archetypes.add_component(src_archetype_idx, type_idx) {
            (type_idx, dst_archetype_idx)
        } else {
            (type_idx, src_archetype_idx)
        }
    }

    pub(crate) fn create_entity(
        &mut self,
        archetype_idx: ArchetypeIdx,
        parent_idx: Option<EntityIdx>,
    ) -> (EntityIdx, EntityLocation) {
        let location = EntityLocation {
            idx: archetype_idx,
            pos: self.archetypes.next_entity_pos(archetype_idx),
        };
        let entity_idx = self.entities.create(location, parent_idx);
        let location = EntityLocation {
            idx: archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, archetype_idx),
        };
        (entity_idx, location)
    }

    pub(crate) fn add_component<C>(
        &mut self,
        component: C,
        type_idx: ComponentTypeIdx,
        location: EntityLocation,
        is_singleton: bool,
    ) where
        C: Any + Sync + Send,
    {
        self.components
            .add(type_idx, location, component, is_singleton);
    }

    pub(crate) fn move_entity(
        &mut self,
        src_location: EntityLocation,
        dst_archetype_idx: ArchetypeIdx,
    ) -> EntityLocation {
        let entity_idx = self.archetypes.entity_idxs(src_location.idx)[src_location.pos];
        let dst_type_idxs = self.archetypes.sorted_type_idxs(dst_archetype_idx);
        for &src_type_idx in self.archetypes.sorted_type_idxs(src_location.idx) {
            if dst_type_idxs.binary_search(&src_type_idx).is_ok() {
                self.components
                    .move_(src_type_idx, src_location, dst_archetype_idx);
            } else {
                self.components.delete(src_type_idx, src_location);
            }
        }
        self.archetypes.delete_entity(src_location);
        let dst_location = EntityLocation {
            idx: dst_archetype_idx,
            pos: self.archetypes.add_entity(entity_idx, dst_archetype_idx),
        };
        self.entities.set_location(entity_idx, dst_location);
        Self::update_moved_entity_location(src_location, &self.archetypes, &mut self.entities);
        dst_location
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: EntityIdx) {
        self.entities.delete(entity_idx, |e, l| {
            for &type_idx in self.archetypes.sorted_type_idxs(l.idx) {
                self.components.delete(type_idx, l);
            }
            self.archetypes.delete_entity(l);
            Self::update_moved_entity_location(l, &self.archetypes, e);
        });
    }

    pub(crate) fn add_system(
        &mut self,
        wrapper: SystemWrapper,
        properties: SystemProperties,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> ActionIdx {
        let action_idx = self.actions.idx_or_create(action_type, action_dependencies);
        self.actions.add_system(action_idx);
        self.systems.add(wrapper, properties, action_idx);
        action_idx
    }

    // This is a workaround for App::with_update.
    // This should be generalized when Filter<F> system param will be added.
    pub(crate) fn run_system_once<S>(&mut self, mut wrapper: S, properties: SystemProperties)
    where
        S: FnMut(SystemData<'_>, SystemInfo<'_>),
    {
        let data = SystemData {
            entities: &self.entities,
            components: &self.components,
            archetypes: &self.archetypes,
            actions: &self.actions,
            updates: &self.updates,
        };
        wrapper(
            data,
            SystemInfo {
                filtered_component_type_idxs: &properties.filtered_component_type_idxs,
                item_count: 1, // generalized: data.item_count(&properties.filtered_component_type_idxs)
            },
        );
    }

    pub(crate) fn update(&mut self) {
        let data = SystemData {
            entities: &self.entities,
            components: &self.components,
            archetypes: &self.archetypes,
            actions: &self.actions,
            updates: &self.updates,
        };
        self.systems.run(data);
        let mut updates = mem::take(
            self.updates
                .get_mut()
                .expect("internal error: cannot access to entity actions"),
        );
        // Each type of update is executed in an order that avoids entity index conflicts
        for (entity_idx, add_component_fns, deleted_component_type_idxs) in
            updates.changed_entity_drain()
        {
            if let Some(location) = self.entities.location(entity_idx) {
                let mut dst_archetype_idx = location.idx;
                for type_idx in deleted_component_type_idxs {
                    dst_archetype_idx = self.delete_component_type(type_idx, dst_archetype_idx);
                }
                for add_fns in &add_component_fns {
                    dst_archetype_idx = (add_fns.add_type_fn)(self, dst_archetype_idx);
                }
                let dst_location = self.move_entity(location, dst_archetype_idx);
                for add_fns in add_component_fns {
                    (add_fns.add_fn)(self, dst_location);
                }
            }
        }
        for (create_fn, parent_idx) in updates.created_child_entity_drain() {
            if self.entities.location(parent_idx).is_some() {
                create_fn(self);
            }
        }
        for create_fn in updates.created_root_entity_drain() {
            create_fn(self);
        }
        for entity_idx in updates.deleted_entity_drain() {
            if self.entities.location(entity_idx).is_some() {
                self.delete_entity(entity_idx);
            }
        }
    }

    fn delete_component_type(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        if let Ok(dst_archetype_idx) = self
            .archetypes
            .delete_component(src_archetype_idx, type_idx)
        {
            dst_archetype_idx
        } else {
            src_archetype_idx
        }
    }

    fn update_moved_entity_location(
        location: EntityLocation,
        archetypes: &ArchetypeStorage,
        entities: &mut EntityStorage,
    ) {
        let archetype_entity_idxs = archetypes.entity_idxs(location.idx);
        if let Some(&moved_entity_idx) = archetype_entity_idxs.get(location.pos) {
            entities.set_location(moved_entity_idx, location);
        }
    }
}
