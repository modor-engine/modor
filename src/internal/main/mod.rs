use crate::internal::actions::ActionFacade;
use crate::internal::core::CoreFacade;
use crate::internal::entity_actions::data::AddComponentFn;
use crate::internal::group_actions::data::{BuildGroupFn, CreateEntityFn};
use crate::internal::system::data::SystemDetails;
use crate::internal::system::SystemFacade;
use crate::{GroupBuilder, SystemData};
use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use std::sync::Mutex;

#[derive(Default)]
pub(crate) struct MainFacade {
    core: CoreFacade,
    systems: SystemFacade,
    actions: Mutex<ActionFacade>,
}

impl MainFacade {
    pub(crate) fn create_group(&mut self) -> NonZeroUsize {
        self.core.create_group()
    }

    pub(crate) fn add_entity_main_component<C>(&mut self) -> bool
    where
        C: Any,
    {
        self.core.add_entity_main_component_type::<C>()
    }

    pub(crate) fn create_entity(&mut self, group_idx: NonZeroUsize) -> usize {
        self.core.create_entity(group_idx)
    }

    pub(crate) fn add_component<C>(&mut self, entity_idx: usize, component: C)
    where
        C: Any + Sync + Send,
    {
        self.core.add_component(entity_idx, component);
    }

    pub(crate) fn add_system(&mut self, group_idx: Option<NonZeroUsize>, system: SystemDetails) {
        self.systems.add(group_idx, system)
    }

    pub(crate) fn run_systems(&mut self) {
        let data = SystemData::new(&self.core, &self.actions);
        self.systems.run(data);
    }

    pub(crate) fn system_data(&self) -> SystemData<'_> {
        SystemData::new(&self.core, &self.actions)
    }

    pub(crate) fn apply_system_actions(&mut self) {
        let result = self
            .actions
            .try_lock()
            .expect("internal error: reset locked actions")
            .reset();
        self.apply_entity_deletions(result.deleted_entity_idxs);
        self.apply_entity_creations(result.entity_builders);
        self.apply_component_deletion(result.deleted_component_types);
        self.apply_component_adds(result.component_adders);
        self.apply_group_deletions(result.deleted_group_idxs);
        self.apply_group_replacements(result.replaced_group_builders);
    }

    pub(crate) fn thread_count(&self) -> u32 {
        self.systems.thread_count()
    }

    pub(crate) fn set_thread_count(&mut self, count: u32) {
        self.systems.set_thread_count(count)
    }

    fn delete_group(&mut self, group_idx: NonZeroUsize) {
        self.core.delete_group(group_idx);
        self.systems.delete_group(group_idx);
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
            self.core.delete_entity(deleted_entity_idx);
        }
    }

    fn apply_component_adds(&mut self, component_adders: Vec<AddComponentFn>) {
        for add_component_fn in component_adders {
            add_component_fn(self);
        }
    }

    fn apply_component_deletion(&mut self, deleted_component_types: Vec<(usize, TypeId)>) {
        for (entity_idx, component_type) in deleted_component_types {
            self.core.delete_component(entity_idx, component_type);
        }
    }
}

#[cfg(test)]
mod main_facade_tests {
    use super::*;
    use crate::external::systems::building::internal::TypeAccess;
    use crate::{Built, EntityBuilder, EntityMainComponent, SystemData, SystemWrapper};
    use std::convert::TryInto;
    use std::fmt::Debug;

    #[derive(PartialEq, Eq, Debug)]
    struct MainComponentType(u32);

    impl EntityMainComponent for MainComponentType {
        type Data = u32;

        fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn create_group() {
        let mut facade = MainFacade::default();

        let group_idx = facade.create_group();

        assert_eq!(group_idx, 1.try_into().unwrap());
        assert_eq!(facade.core.create_group(), 2.try_into().unwrap());
    }

    #[test]
    fn add_entity_main_component_type() {
        let mut facade = MainFacade::default();

        let new_type = facade.add_entity_main_component::<u32>();

        assert!(new_type);
        assert!(!facade.core.add_entity_main_component_type::<u32>());
    }

    #[test]
    fn create_entity() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();

        let entity_idx = facade.create_entity(group_idx);

        assert_eq!(entity_idx, 0);
        assert_eq!(facade.core.create_entity(group_idx), 1);
    }

    #[test]
    fn add_component_to_entity() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);

        facade.add_component(entity_idx, 42_u32);

        assert_eq!(facade.core.archetype_entity_idxs(0), &[entity_idx]);
        let components = facade.core.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![&42]));
    }

    #[test]
    fn add_system() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();

        facade.add_system(
            Some(group_idx),
            SystemDetails::new(
                |d, i| {
                    d.actions_mut().delete_entity(10);
                    assert_eq!(i.group_idx, Some(1.try_into().unwrap()));
                    assert_eq!(i.filtered_component_types, [TypeId::of::<i64>()])
                },
                vec![TypeAccess::Read(TypeId::of::<u32>())],
                Some(TypeId::of::<i64>()),
                true,
            ),
        );

        let data = SystemData::new(&facade.core, &facade.actions);
        facade.systems.run(data);
        let action_result = facade.actions.get_mut().unwrap().reset();
        assert_eq!(action_result.deleted_entity_idxs, [10]);
    }

    #[test]
    fn run_systems() {
        let mut facade = MainFacade::default();
        let wrapper: SystemWrapper = |d, _| d.actions_mut().delete_entity(10);
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));

        facade.run_systems();

        let action_result = facade.actions.get_mut().unwrap().reset();
        assert_eq!(action_result.deleted_entity_idxs, [10]);
    }

    #[test]
    fn retrieve_system_data() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity = facade.create_entity(group_idx);
        facade.add_component(entity, 42_u32);

        let data = facade.system_data();

        let components = data.read_components::<u32>().unwrap();
        let component_iter = components.0.archetype_iter(0);
        assert_option_iter!(component_iter, Some(vec![&42]));
    }

    #[test]
    fn apply_entity_deletion_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component::<u32>(entity_idx, 42);
        facade.add_component::<i64>(entity_idx, 13);
        let wrapper: SystemWrapper = |d, _| d.actions_mut().delete_entity(0);
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));
        facade.run_systems();

        facade.apply_system_actions();

        assert_eq!(facade.core.create_entity(group_idx), entity_idx);
        let components = facade.core.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        let components = facade.core.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
    }

    #[test]
    fn apply_entity_creation_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let wrapper: SystemWrapper = |d, _| {
            let group_idx = 1.try_into().unwrap();
            let create_fn: CreateEntityFn = Box::new(move |m| {
                m.create_entity(group_idx);
            });
            d.actions_mut().create_entity(group_idx, create_fn);
        };
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));
        facade.run_systems();

        facade.apply_system_actions();

        assert_eq!(facade.core.create_entity(group_idx), 1);
    }

    #[test]
    fn apply_component_deletion_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity1_idx = facade.create_entity(group_idx);
        facade.add_component::<u32>(entity1_idx, 10);
        facade.add_component::<i64>(entity1_idx, 20);
        let wrapper: SystemWrapper = |d, _| d.actions_mut().delete_component::<u32>(0);
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));
        facade.run_systems();

        facade.apply_system_actions();

        let components = facade.core.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        let components = facade.core.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        assert_option_iter!(components.archetype_iter(2), Some(vec![&20]));
    }

    #[test]
    fn apply_component_add_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        facade.create_entity(group_idx);
        let wrapper: SystemWrapper = |d, _| {
            let add_fn: AddComponentFn = Box::new(|m| m.add_component::<u32>(0, 10));
            d.actions_mut().add_component(0, add_fn);
        };
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));
        facade.run_systems();

        facade.apply_system_actions();

        let components = facade.core.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![&10]));
    }

    #[test]
    fn apply_group_deletion_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        facade.add_component::<u32>(entity_idx, 42);
        facade.add_component::<i64>(entity_idx, 13);
        facade.create_entity(group_idx);
        let wrapper1: SystemWrapper = |d, _| d.actions_mut().delete_group(1.try_into().unwrap());
        facade.add_system(
            Some(group_idx),
            SystemDetails::new(wrapper1, Vec::new(), None, true),
        );
        let wrapper2: SystemWrapper = |d, _| d.actions_mut().delete_entity(1);
        facade.add_system(
            Some(group_idx),
            SystemDetails::new(wrapper2, Vec::new(), None, true),
        );
        facade.run_systems();

        facade.apply_system_actions();

        assert_eq!(facade.core.create_group(), group_idx);
        let data = SystemData::new(&facade.core, &facade.actions);
        facade.systems.run(data);
        let action_result = facade.actions.get_mut().unwrap().reset();
        assert_eq!(action_result.deleted_entity_idxs, []);
        let components = facade.core.read_components::<u32>().unwrap();
        assert_option_iter!(components.archetype_iter(0), Some(vec![]));
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
        let components = facade.core.read_components::<i64>().unwrap();
        assert_option_iter!(components.archetype_iter(1), Some(vec![]));
    }

    #[test]
    fn apply_group_replacement_system_action() {
        let mut facade = MainFacade::default();
        let group_idx = facade.create_group();
        let entity_idx = facade.create_entity(group_idx);
        let wrapper: SystemWrapper = |d, _| {
            let build_fn: BuildGroupFn = Box::new(|builder| {
                builder.with_entity::<MainComponentType>(10);
            });
            d.actions_mut()
                .replace_group(1.try_into().unwrap(), build_fn)
        };
        facade.add_system(None, SystemDetails::new(wrapper, Vec::new(), None, true));
        facade.run_systems();

        facade.apply_system_actions();

        assert_eq!(facade.core.create_group(), 2.try_into().unwrap());
        assert_eq!(facade.core.create_entity(group_idx), entity_idx + 1);

        let components = facade.core.read_components::<MainComponentType>().unwrap();
        let expected = vec![&MainComponentType(10)];
        assert_option_iter!(components.archetype_iter(0), Some(expected));
    }

    #[test]
    fn set_thread_count() {
        let mut facade = MainFacade::default();

        facade.set_thread_count(2);

        assert_eq!(facade.systems.thread_count(), 2);
    }

    #[test]
    fn retrieve_thread_count() {
        let mut facade = MainFacade::default();
        facade.set_thread_count(2);

        let thread_count = facade.thread_count();

        assert_eq!(thread_count, 2);
    }
}
