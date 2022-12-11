use std::any::{Any, TypeId};
use std::mem;
use std::sync::{Mutex, RwLock};

use super::archetype_states::ArchetypeStateStorage;
use super::systems::FullSystemProperties;
use crate::storages::actions::{ActionDependencies, ActionIdx, ActionStorage};
use crate::storages::archetypes::{ArchetypeIdx, ArchetypeStorage, EntityLocation};
use crate::storages::components::{ComponentStorage, ComponentTypeIdx};
use crate::storages::entities::{EntityIdx, EntityStorage};
use crate::storages::systems::SystemStorage;
use crate::storages::updates::UpdateStorage;
use crate::systems::context::{Storages, SystemContext};
use crate::SystemBuilder;

#[derive(Default)]
pub struct CoreStorage {
    archetypes: ArchetypeStorage,
    entities: EntityStorage,
    components: ComponentStorage,
    actions: ActionStorage,
    systems: SystemStorage,
    updates: Mutex<UpdateStorage>,
    archetype_states: RwLock<ArchetypeStateStorage>,
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
        self.archetypes
            .add_component(src_archetype_idx, type_idx, TypeId::of::<C>())
            .map_or((type_idx, src_archetype_idx), |dst_archetype_idx| {
                (type_idx, dst_archetype_idx)
            })
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
        properties: FullSystemProperties,
        action_type: Option<TypeId>,
        action_dependencies: ActionDependencies,
    ) -> ActionIdx {
        let label = properties.label;
        let action_idx = self.actions.idx_or_create(action_type, action_dependencies);
        self.actions.add_system(action_idx);
        let mutation_component_type_idxs = properties.mutation_component_type_idxs.clone();
        let system_idx = self.systems.add(properties, action_idx);
        self.archetype_states
            .get_mut()
            .expect("internal error: cannot add system in archetype state")
            .add_system(system_idx, &mutation_component_type_idxs);
        debug!("system `{label}` initialized");
        action_idx
    }

    pub(crate) fn run_system<S>(&mut self, mut system: SystemBuilder<S>)
    where
        S: FnMut(SystemContext<'_>),
    {
        let _properties = (system.properties_fn)(self); // to create component types
        let storages = Storages {
            entities: &self.entities,
            components: &self.components,
            archetypes: &self.archetypes,
            actions: &self.actions,
            updates: &self.updates,
            archetype_states: &self.archetype_states,
        };
        (system.wrapper)(SystemContext {
            system_idx: None,
            archetype_filter_fn: system.archetype_filter_fn,
            entity_type_idx: None,
            item_count: storages.item_count(None, system.archetype_filter_fn, None),
            storages,
        });
    }

    pub(crate) fn update(&mut self) {
        let data = Storages {
            entities: &self.entities,
            components: &self.components,
            archetypes: &self.archetypes,
            actions: &self.actions,
            updates: &self.updates,
            archetype_states: &self.archetype_states,
        };
        self.systems.run(data);
        self.archetypes.reset_state();
        self.entities.reset_state();
        self.archetype_states
            .get_mut()
            .expect("internal error: cannot reset archetype state")
            .reset_state();
        let mut updates = mem::take(
            self.updates
                .get_mut()
                .expect("internal error: cannot access to entity actions"),
        );
        // Each type of update is executed in an order that avoids entity index conflicts
        for (entity_idx, add_component_fns, deleted_component_type_idxs) in
            updates.changed_entity_drain()
        {
            self.entities.location(entity_idx).map_or_else(
                || {
                    warn!(
                        "components cannot be modified as entity with ID {} doesn't exist",
                        entity_idx.0
                    );
                },
                |location| {
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
                },
            );
        }
        for (create_fn, parent_idx) in updates.created_child_entity_drain() {
            if self.entities.location(parent_idx).is_some() {
                create_fn(self);
            } else {
                warn!(
                    "child entity not created as parent entity with ID {} doesn't exist",
                    parent_idx.0
                );
            }
        }
        for create_fn in updates.created_root_entity_drain() {
            create_fn(self);
        }
        for entity_idx in updates.deleted_entity_drain() {
            if self.entities.location(entity_idx).is_some() {
                self.delete_entity(entity_idx);
                trace!("entity with ID {} deleted", entity_idx.0);
            } else {
                warn!(
                    "entity with ID {} not deleted as it doesn't exist",
                    entity_idx.0
                );
            }
        }
    }

    fn delete_component_type(
        &mut self,
        type_idx: ComponentTypeIdx,
        src_archetype_idx: ArchetypeIdx,
    ) -> ArchetypeIdx {
        self.archetypes
            .delete_component(src_archetype_idx, type_idx)
            .map_or(src_archetype_idx, |dst_archetype_idx| dst_archetype_idx)
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
