use crate::internal::components::ComponentFacade;
use crate::internal::core::CoreFacade;
use crate::internal::entity_actions::EntityActionFacade;
use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use crate::internal::group_actions::GroupActionFacade;
use crate::internal::system::data::SystemDetails;
use crate::internal::system::SystemFacade;
use crate::GroupBuilder;
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct MainFacade {
    core: CoreFacade,
    components: ComponentFacade,
    systems: SystemFacade,
    // TODO: move in new ActionFacade type
    group_actions: Mutex<GroupActionFacade>,
    entity_actions: Mutex<EntityActionFacade>,
}

impl MainFacade {
    /// Return group index.
    pub(crate) fn create_group(&mut self) -> NonZeroUsize {
        self.core.create_group()
    }

    pub(crate) fn delete_group(&mut self, group_idx: NonZeroUsize) {
        for type_idxs in self.core.group_component_type_idxs(group_idx) {
            for &archetype_idx in self.core.group_archetype_idxs(group_idx) {
                self.components.delete_archetype(type_idxs, archetype_idx);
            }
        }
        self.core.delete_group(group_idx);
        self.systems.delete_group(group_idx);
    }

    /// Return whether the type is new for the group.
    pub(crate) fn add_entity_type(&mut self, group_idx: NonZeroUsize, entity_type: TypeId) -> bool {
        self.core.add_entity_type(group_idx, entity_type)
    }

    /// Return entity index.
    pub(crate) fn create_entity(&mut self, group_idx: NonZeroUsize) -> usize {
        self.core.create_entity(group_idx)
    }

    pub(crate) fn delete_entity(&mut self, entity_idx: usize) {
        if let Some(location) = self.core.entity_location(entity_idx) {
            for &component_type_idx in self.core.archetype_type_idxs(location.archetype_idx) {
                self.components.swap_delete(component_type_idx, location);
            }
        }
        self.core.delete_entity(entity_idx);
    }

    pub(crate) fn add_component<C>(&mut self, entity_idx: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        let type_idx = self
            .core
            .component_type_idx(TypeId::of::<C>())
            .map_or_else(|| self.create_component_type::<C>(), |type_idx| type_idx);
        let location = self.core.entity_location(entity_idx);
        if let Some(location) = location {
            if self.components.exists::<C>(type_idx, location) {
                self.components.replace(type_idx, location, component);
            } else {
                let new_archetype_idx = self.core.add_component(entity_idx, type_idx);
                for &moved_type_idx in self.core.archetype_type_idxs(location.archetype_idx) {
                    self.components
                        .move_(moved_type_idx, location, new_archetype_idx);
                }
                self.components.add(type_idx, new_archetype_idx, component);
            }
        } else {
            let new_archetype_idx = self.core.add_component(entity_idx, type_idx);
            self.components.add(type_idx, new_archetype_idx, component);
        }
    }

    pub(crate) fn delete_component<C>(&mut self, entity_idx: usize)
    where
        C: Any,
    {
        let type_idx = self.core.component_type_idx(TypeId::of::<C>()).unwrap();
        let location = self.core.entity_location(entity_idx).unwrap();
        if let Some(new_archetype_idx) = self.core.delete_component(entity_idx, type_idx) {
            for &moved_type_idx in self.core.archetype_type_idxs(location.archetype_idx) {
                self.components
                    .move_(moved_type_idx, location, new_archetype_idx);
            }
        } else {
            for &deleted_type_idx in self.core.archetype_type_idxs(location.archetype_idx) {
                self.components.swap_delete(deleted_type_idx, location);
            }
        }
        self.components.swap_delete(type_idx, location);
    }

    pub(crate) fn add_system(&mut self, group_idx: Option<NonZeroUsize>, system: SystemDetails) {
        self.systems.add(group_idx, system)
    }

    pub(crate) fn run_systems(&mut self) {
        self.systems.run(
            &self.core,
            &self.components.components(),
            &self.group_actions,
            &self.entity_actions,
        );
    }

    pub(crate) fn apply_system_actions(&mut self) {
        self.apply_group_actions();
        self.apply_entity_actions();
    }

    pub(crate) fn set_thread_count(&mut self, count: u32) {
        self.systems.set_thread_count(count)
    }

    fn create_component_type<C>(&mut self) -> usize
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.core.add_component_type(TypeId::of::<C>());
        self.components.create_type::<C>();
        type_idx
    }

    fn apply_group_actions(&mut self) {
        let actions = self.group_actions.get_mut().unwrap();
        let deleted_group_idxs: Vec<_> = actions.deleted_group_idxs().collect();
        let replaced_group_builders: Vec<_> = actions.replaced_group_builders().collect();
        let entity_builders: Vec<_> = actions.entity_builders().collect();
        actions.reset();
        self.apply_group_deletions(deleted_group_idxs);
        self.apply_group_replacements(replaced_group_builders);
        self.apply_entity_creations(entity_builders)
    }

    fn apply_entity_actions(&mut self) {
        let actions = self.entity_actions.get_mut().unwrap();
        let deleted_entity_idxs: Vec<_> = actions.deleted_entity_idxs().collect();
        actions.reset();
        self.apply_entity_deletions(deleted_entity_idxs);
    }

    fn apply_entity_creations(&mut self, entity_builders: Vec<CreateEntityFn>) {
        for entity_builder in entity_builders {
            entity_builder(self);
        }
    }

    fn apply_group_replacements(
        &mut self,
        replaced_group_builders: Vec<(NonZeroUsize, BuildGroupFn)>,
    ) {
        for (replaced_group_idx, group_builder_fn) in replaced_group_builders {
            self.delete_group(replaced_group_idx);
            let new_group_idx = self.create_group();
            group_builder_fn(&mut GroupBuilder::new(self, new_group_idx));
        }
    }

    fn apply_group_deletions(&mut self, deleted_group_idxs: Vec<NonZeroUsize>) {
        for deleted_group_idx in deleted_group_idxs {
            self.delete_group(deleted_group_idx);
        }
    }

    fn apply_entity_deletions(&mut self, deleted_entity_idxs: Vec<usize>) {
        for deleted_entity_idx in deleted_entity_idxs {
            self.delete_entity(deleted_entity_idx);
        }
    }
}
