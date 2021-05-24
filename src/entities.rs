use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::SystemBuilder;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::num::NonZeroUsize;

/// Define main component for entities.
///
/// This component type is used to instanciate entities. It can be queried by systems like any
/// other component of the entity.<br>
/// Entity systems only iterate on entities containing the corresponding main component type.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
/// # };
/// #
/// struct Body;
///
/// impl EntityMainComponent for Body {
///     type Data = (f32, f32);
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder
///             .with(Position { x: data.0, y: data.1 })
///             .with(Velocity { x: 2., y: 5. })
///             .with_self(Self)
///     }
///
///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
///         runner.run(system!(Self::update_position));
///     }
/// }
///
/// impl Body {
///     fn update_position(position: &mut Position, velocity: &Velocity) {
///         position.x += velocity.x;
///         position.y += velocity.y;
///     }
/// }
/// #
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// #
/// # struct Velocity {
/// #     x: f32,
/// #     y: f32,
/// # }
/// ```
pub trait EntityMainComponent: Sized + Any + Sync + Send {
    /// Type of data provided to build a new entity.
    type Data: Any + Sync + Send;

    /// Build an entity.
    ///
    /// `builder` is used to define the entity, like adding a component or a system for the entity.
    /// <br>
    /// `data` can be used to customize the creation of the entities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = (f32, f32);
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data.0, y: data.1 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    /// }
    /// #
    /// # struct Position {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// #
    /// # struct Velocity {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// ```
    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built;

    /// Define systems that are run for the entity.
    ///
    /// `runner` is used to define which systems are run and how.
    ///
    /// This method is only called once, at the same time as the creation of the first entity of
    /// this type, and applied to all entities created with this type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = (f32, f32);
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data.0, y: data.1 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    ///
    ///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
    ///         runner
    ///             .run(system!(Self::update_position))
    ///             .run(system!(Self::increment_velocity));
    ///     }
    /// }
    ///
    /// impl Body {
    ///     fn update_position(position: &mut Position, velocity: &Velocity) {
    ///         position.x += velocity.x;
    ///         position.y += velocity.y;
    ///     }
    ///
    ///     fn increment_velocity(velocity: &mut Velocity) {
    ///         velocity.x += 1.;
    ///         velocity.y += 1.;
    ///     }
    /// }
    /// #
    /// # struct Position {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// #
    /// # struct Velocity {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// ```
    #[allow(unused_variables)]
    fn on_update(runner: &mut EntityRunner<'_, Self>) {}
}

/// Interface to build an entity.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
/// # };
/// #
/// struct Body;
///
/// impl EntityMainComponent for Body {
///     type Data = (f32, f32);
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder
///             .with(Position { x: data.0, y: data.1 })
///             .with(Velocity { x: 2., y: 5. })
///             .with_self(Self)
///     }
/// }
/// #
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// #
/// # struct Velocity {
/// #     x: f32,
/// #     y: f32,
/// # }
/// ```
pub struct EntityBuilder<'a, M> {
    main: &'a mut MainFacade,
    entity_idx: usize,
    group_idx: NonZeroUsize,
    phantom: PhantomData<M>,
}

impl<'a, M> EntityBuilder<'a, M>
where
    M: EntityMainComponent,
{
    /// Indicate that the built entity inherits from a parent entity.
    ///
    /// It means components and systems of the parent entity are applied to the built one.<br>
    /// If the parent has a component with a type already added to the built entity, the parent's
    /// component overwrites the exisiting one.<br>
    /// If after calling this method, a component with a type contained in the parent entity is
    /// added to the built entity, the new component overwrites the parent's.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Parent;
    ///
    /// impl EntityMainComponent for Parent {
    ///     type Data = String;
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(String::from(data))
    ///             .with_self(Self)
    ///     }
    ///
    ///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
    ///         runner.run(system!(Self::parent_system));
    ///     }
    /// }
    ///
    /// impl Parent {
    ///     // Iterates on entities containing both
    ///     // - String component (because of system args)
    ///     // - Parent component (because specified in EntityMainComponent::on_update)
    ///     // Because Child inherit from Parent, the system iterates on Child entities too.
    ///     fn parent_system(label: &mut String) {
    ///         *label = format!("modified label (initial: {})", label);
    ///     }
    /// }
    ///
    /// struct Child;
    ///
    /// impl EntityMainComponent for Child {
    ///     type Data = String;
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .inherit_from::<Parent>(data)
    ///             .with(42_u32)
    ///             .with_self(Self)
    ///         // child entity contains components of type Parent, String, Child and u32
    ///     }
    /// }
    /// ```
    pub fn inherit_from<P>(&mut self, data: P::Data) -> &mut Self
    where
        P: EntityMainComponent,
    {
        let mut entity_builder = EntityBuilder::new(self.main, self.entity_idx, self.group_idx);
        P::build(&mut entity_builder, data);
        self
    }

    /// Add a component to the built entity.
    ///
    /// If a component of type `C` already exists for the entity, it is overwritten.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = (f32, f32);
    ///
    ///     // entity contains Position, Velocity and Body components
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data.0, y: data.1 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    /// }
    /// #
    /// # struct Position {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// #
    /// # struct Velocity {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// ```
    pub fn with<C>(&mut self, component: C) -> &mut Self
    where
        C: Any + Sync + Send,
    {
        self.main.add_component(self.entity_idx, component);
        self
    }

    /// Add main component of the built entity.
    ///
    /// If the component type already exists for the entity, it is overwritten.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = (f32, f32);
    ///
    ///     // entity contains Position, Velocity and Body components
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data.0, y: data.1 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    /// }
    /// #
    /// # struct Position {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// #
    /// # struct Velocity {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// ```
    pub fn with_self(&mut self, entity: M) -> Built {
        self.with(entity);
        let new_type = self.main.add_entity_main_component::<M>();
        if new_type {
            M::on_update(&mut EntityRunner::new(self.main));
        }
        Built::new()
    }

    pub(crate) fn new(
        main: &'a mut MainFacade,
        entity_idx: usize,
        group_idx: NonZeroUsize,
    ) -> Self {
        Self {
            main,
            entity_idx,
            group_idx,
            phantom: PhantomData,
        }
    }
}

/// Interface to define systems run for an entity.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
/// # };
/// #
/// struct Body;
///
/// impl EntityMainComponent for Body {
///     type Data = (f32, f32);
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder
///             .with(Position { x: data.0, y: data.1 })
///             .with(Velocity { x: 2., y: 5. })
///             .with_self(Self)
///     }
///
///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
///         runner
///             .run(system!(Self::update_position))
///             .run(system!(Self::increment_velocity));
///     }
/// }
///
/// impl Body {
///     fn update_position(position: &mut Position, velocity: &Velocity) {
///         position.x += velocity.x;
///         position.y += velocity.y;
///     }
///
///     fn increment_velocity(velocity: &mut Velocity) {
///         velocity.x += 1.;
///         velocity.y += 1.;
///     }
/// }
/// #
/// # struct Position {
/// #     x: f32,
/// #     y: f32,
/// # }
/// #
/// # struct Velocity {
/// #     x: f32,
/// #     y: f32,
/// # }
/// ```
pub struct EntityRunner<'a, M> {
    main: &'a mut MainFacade,
    phantom: PhantomData<M>,
}

impl<'a, M> EntityRunner<'a, M>
where
    M: EntityMainComponent,
{
    /// Add a system for the entities.
    ///
    /// Systems are functions or closures able to iterate on entities to update their
    /// components, or to run actions like deleting an entity, creating a group, ...<br>
    /// Execution order of systems is undefined.
    ///
    /// Systems registered by this method are run each time
    /// [`Application::update`](crate::Application::update) is called, and iterates on all queried
    /// entities containing a component of type `M` regardless their group.<br>
    /// [`Query`](crate::Query) arguments of entity systems iterate on all queried entities
    /// regardless the entity group and type.
    ///
    /// `system` must be defined using the [`system!`](crate::system!) macro.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
    /// # };
    /// #
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = (f32, f32);
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data.0, y: data.1 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    ///
    ///     fn on_update(runner: &mut EntityRunner<'_, Self>) {
    ///         runner
    ///             .run(system!(Self::update_position))
    ///             .run(system!(Self::increment_velocity));
    ///     }
    /// }
    ///
    /// impl Body {
    ///     fn update_position(position: &mut Position, velocity: &Velocity) {
    ///         position.x += velocity.x;
    ///         position.y += velocity.y;
    ///     }
    ///
    ///     fn increment_velocity(velocity: &mut Velocity) {
    ///         velocity.x += 1.;
    ///         velocity.y += 1.;
    ///     }
    /// }
    /// #
    /// # struct Position {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// #
    /// # struct Velocity {
    /// #     x: f32,
    /// #     y: f32,
    /// # }
    /// ```
    pub fn run(&mut self, system: SystemBuilder) -> &mut Self {
        let entity_type = Some(TypeId::of::<M>());
        let system = SystemDetails::new(
            system.wrapper,
            system.component_types,
            entity_type,
            system.actions,
        );
        self.main.add_system(None, system);
        self
    }

    fn new(ecs: &'a mut MainFacade) -> Self {
        Self {
            main: ecs,
            phantom: PhantomData,
        }
    }
}

/// Type that ensures entities are correctly built.
///
/// This is done by ensuring [`EntityBuilder::with_self`](crate::EntityBuilder::with_self)
/// has been called at least once.
///
/// # Examples
///
/// ```rust
/// # use modor::{
/// #     system, EntityMainComponent, Built, EntityBuilder, GroupBuilder, EntityRunner
/// # };
/// #
/// struct MyEntity(String);
///
/// impl EntityMainComponent for MyEntity {
///     type Data = String;
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder
///             .with(42_u32)
///             .with_self(Self(data))
///     }
/// }
/// ```
pub struct Built(PhantomData<()>);

impl Built {
    fn new() -> Self {
        Self(PhantomData)
    }
}

#[cfg(test)]
mod entity_builder_tests {
    use super::*;
    use crate::SystemOnceBuilder;

    assert_impl_all!(EntityBuilder<'_, String>: Send);
    assert_not_impl_any!(EntityBuilder<'_, String>: Clone);

    #[derive(PartialEq, Eq, Debug)]
    struct Parent;

    impl EntityMainComponent for Parent {
        type Data = String;

        fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with(data).with(42_u32).with_self(Self)
        }
    }

    #[derive(PartialEq, Eq, Debug)]
    struct Child;

    impl EntityMainComponent for Child {
        type Data = ();

        fn build(builder: &mut EntityBuilder<'_, Self>, _: Self::Data) -> Built {
            builder.with_self(Self)
        }
    }

    #[test]
    fn make_entity_inherit_from_another_one() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity_idx = main.create_entity(group_idx);
        let mut builder = EntityBuilder::<Child>::new(&mut main, entity_idx, group_idx);

        builder.inherit_from::<Parent>("text".into());

        assert!(!main.add_entity_main_component::<Parent>());
        main.run_system_once(SystemOnceBuilder::new(|d, _| {
            let components = d.read_components::<u32>().unwrap();
            let component_iter = d.component_iter::<u32>(&components.0, 2);
            assert_option_iter!(component_iter, Some(vec![&42]));
            let components = d.read_components::<String>().unwrap();
            let component_iter = d.component_iter::<String>(&components.0, 2);
            assert_option_iter!(component_iter, Some(vec![&String::from("text")]));
            let components = d.read_components::<Parent>().unwrap();
            let component_iter = d.component_iter::<Parent>(&components.0, 2);
            assert_option_iter!(component_iter, Some(vec![&Parent]));
        }));
    }

    #[test]
    fn add_component_to_entity() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity_idx = main.create_entity(group_idx);
        let mut builder = EntityBuilder::<Child>::new(&mut main, entity_idx, group_idx);

        builder.with::<String>("text".into());

        main.run_system_once(SystemOnceBuilder::new(|d, _| {
            let components = d.read_components::<String>().unwrap();
            let component_iter = d.component_iter::<String>(&components.0, 0);
            assert_option_iter!(component_iter, Some(vec![&String::from("text")]));
        }));
    }

    #[test]
    fn add_main_component_to_entity() {
        let mut main = MainFacade::default();
        let group_idx = main.create_group();
        let entity_idx = main.create_entity(group_idx);
        let mut builder = EntityBuilder::<Child>::new(&mut main, entity_idx, group_idx);

        builder.with_self(Child);

        assert!(!main.add_entity_main_component::<Child>());
        main.run_system_once(SystemOnceBuilder::new(|d, _| {
            let components = d.read_components::<Child>().unwrap();
            let component_iter = d.component_iter::<Child>(&components.0, 0);
            assert_option_iter!(component_iter, Some(vec![&Child]));
        }));
    }
}

#[cfg(test)]
mod entity_runner_tests {
    use super::*;
    use std::convert::TryInto;

    assert_impl_all!(EntityRunner<'_, String>: Send);
    assert_not_impl_any!(EntityRunner<'_, String>: Clone);

    #[derive(PartialEq, Eq, Debug)]
    struct MyEntity;

    impl EntityMainComponent for MyEntity {
        type Data = ();

        fn build(builder: &mut EntityBuilder<'_, Self>, _: Self::Data) -> Built {
            builder.with_self(Self)
        }
    }

    #[test]
    fn add_entity_system() {
        let mut main = MainFacade::default();
        main.create_group();
        let mut builder = EntityRunner::<MyEntity>::new(&mut main);

        builder.run(SystemBuilder::new(
            |d, i| {
                d.actions_mut().delete_group(1.try_into().unwrap());
                assert_eq!(i.filtered_component_types, vec![TypeId::of::<MyEntity>()]);
                assert_eq!(i.group_idx, None);
            },
            vec![],
            true,
        ));

        main.run_systems();
        main.apply_system_actions();
        assert_eq!(main.create_group(), 1.try_into().unwrap());
    }
}

#[cfg(test)]
mod built_tests {
    use super::*;

    assert_impl_all!(Built: Sync, Send);
    assert_not_impl_any!(Built: Clone);
}
