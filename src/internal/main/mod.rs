use crate::internal::components::ComponentFacade;
use crate::internal::core::CoreFacade;
use crate::internal::group_actions::GroupActionFacade;
use crate::internal::system::data::SystemInfo;
use crate::internal::system::SystemFacade;
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct MainFacade {
    core: CoreFacade,
    components: ComponentFacade,
    systems: SystemFacade,
    group_actions: Mutex<GroupActionFacade>,
}

impl MainFacade {
    /// Return group index.
    pub(crate) fn create_group(&mut self) -> NonZeroUsize {
        let group_idx = self.core.create_group();
        self.group_actions_mut().register_group(group_idx);
        group_idx
    }

    #[allow(dead_code)] // TODO: delete
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

    #[allow(dead_code)] // TODO: delete
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

    #[allow(dead_code)] // TODO: delete
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

    pub(crate) fn add_system(&mut self, group_idx: Option<NonZeroUsize>, system: SystemInfo) {
        self.systems.add(group_idx, system)
    }

    pub(crate) fn run_systems(&mut self) {
        self.systems.run(
            &self.core,
            &self.components.components(),
            &self.group_actions,
        );
        // TODO: do something with group actions
    }

    pub(crate) fn set_thread_count(&mut self, count: u32) {
        self.systems.set_thread_count(count)
    }

    fn group_actions_mut(&mut self) -> &mut GroupActionFacade {
        self.group_actions.get_mut().unwrap()
    }

    fn create_component_type<C>(&mut self) -> usize
    where
        C: Any + Sync + Send,
    {
        let type_idx = self.core.add_component_type(TypeId::of::<C>());
        self.components.create_type::<C>();
        type_idx
    }
}
