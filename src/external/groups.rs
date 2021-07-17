use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::{EntityBuilder, EntityMainComponent, SystemBuilder};
use std::num::NonZeroUsize;

/// Interface to build a group of entities.
///
/// # Examples
///
/// ```rust
/// # use modor::{GroupBuilder, EntityMainComponent, EntityBuilder, Built};
/// #
/// fn build_main_menu(builder: &mut GroupBuilder<'_>) {
///     builder
///         .with_entity::<Button>("New game".into())
///         .with_entity::<Button>("Settings".into())
///         .with_entity::<Button>("Exit".into());
/// }
///
/// # struct Button(String);
/// #
/// # impl EntityMainComponent for Button {
/// #     type Data = String;
/// #
/// #     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
/// #         builder.with_self(Self(data))
/// #     }
/// # }
/// ```
pub struct GroupBuilder<'a> {
    main: &'a mut MainFacade,
    group_idx: NonZeroUsize,
}

impl<'a> GroupBuilder<'a> {
    /// Add a system for the group.
    ///
    /// Systems are functions or closures able to iterate on entities to update their
    /// components, or to run actions like deleting an entity, creating a group, ...<br>
    /// Execution order of systems is undefined.
    ///
    /// Systems registered by this method are run each time
    /// [`Application::update`](crate::Application::update) is called, and iterates on all queried
    /// entities that belong to the group.<br>
    /// By default, [`Query`](crate::Query) arguments of group systems also iterate on queried entities
    /// that belong to the group. They can iterate on all entities of the application by calling
    /// [`Query::unlock_groups`](crate::Query::unlock_groups).
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{GroupBuilder, EntityMainComponent, EntityBuilder, Built, system};
    /// #
    /// fn build_version_info(builder: &mut GroupBuilder<'_>, version: String) {
    ///     builder
    ///         .with_entity::<Text>(version)
    ///         .on_update(system!(print_version));
    /// }
    ///
    /// fn print_version(text: &Text) {
    ///     if let Some(version) = text.label.strip_prefix("Version ") {
    ///         println!("Version: {}", version);
    ///     }
    /// }
    /// #
    /// # struct Text {label: String}
    /// #
    /// # impl EntityMainComponent for Text {
    /// #     type Data = String;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    /// #         builder.with_self(Self {label: data})
    /// #     }
    /// # }
    /// ```
    pub fn on_update(&mut self, system: SystemBuilder) -> &mut Self {
        let system =
            SystemDetails::new(system.wrapper, system.component_types, None, system.actions);
        self.main.add_system(Some(self.group_idx), system);
        self
    }

    /// Create an entity in the group.
    ///
    /// If the group is deleted, the entity is also deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{GroupBuilder, EntityMainComponent, EntityBuilder, Built};
    /// #
    /// fn build_main_menu(builder: &mut GroupBuilder<'_>) {
    ///     builder
    ///         .with_entity::<Button>("New game".into())
    ///         .with_entity::<Button>("Settings".into())
    ///         .with_entity::<Button>("Exit".into());
    /// }
    ///
    /// # struct Button(String);
    /// #
    /// # impl EntityMainComponent for Button {
    /// #     type Data = String;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    /// #         builder.with_self(Self(data))
    /// #     }
    /// # }
    /// ```
    pub fn with_entity<M>(&mut self, data: M::Data) -> &mut Self
    where
        M: EntityMainComponent,
    {
        let entity_idx = self.main.create_entity(self.group_idx);
        let mut entity_builder = EntityBuilder::new(self.main, entity_idx, self.group_idx);
        M::build(&mut entity_builder, data);
        self
    }

    pub(crate) fn new(main: &'a mut MainFacade, group_idx: NonZeroUsize) -> Self {
        Self { main, group_idx }
    }
}

#[cfg(test)]
mod group_builder_tests {
    use super::*;
    use crate::Built;
    use std::convert::TryInto;

    assert_impl_all!(GroupBuilder<'_>:  Send);
    assert_not_impl_any!(GroupBuilder<'_>: Clone);

    #[derive(PartialEq, Eq, Debug)]
    struct MyEntity(String);

    impl EntityMainComponent for MyEntity {
        type Data = String;

        fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn add_group_system() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let mut builder = GroupBuilder::new(&mut main, group_idx);

        builder.on_update(SystemBuilder::new(
            |d, i| {
                d.actions_mut().delete_group(1.try_into().unwrap());
                assert_eq!(i.filtered_component_types, Vec::new());
                assert_eq!(i.group_idx, Some(1.try_into().unwrap()));
            },
            vec![],
            true,
        ));

        main.run_systems();
        main.apply_system_actions();
        assert_eq!(main.create_group(), 1.try_into().unwrap());
    }

    #[test]
    fn create_entity_for_group() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let mut builder = GroupBuilder::new(&mut main, group_idx);

        builder.with_entity::<MyEntity>("text".into());

        assert!(!main.add_entity_main_component::<MyEntity>());
        let data = main.system_data();
        let components = data.read_components::<MyEntity>().unwrap();
        let component_iter = components.0.archetype_iter(0);
        assert_option_iter!(component_iter, Some(vec![&MyEntity(String::from("text"))]));
    }
}
