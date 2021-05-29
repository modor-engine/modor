use crate::internal::main::MainFacade;
use crate::internal::system::data::SystemDetails;
use crate::{GroupBuilder, SystemBuilder, SystemData, SystemInfo, SystemOnceBuilder};

/// Entrypoint of Modor.
///
/// # Examples
///
/// ```rust
/// # use modor::{Application, GroupBuilder, EntityMainComponent, EntityBuilder, Built};
/// #
/// Application::new()
///     .with_group(build_main_menu)
///     .update();
///
/// fn build_main_menu(builder: &mut GroupBuilder<'_>) {
///     builder
///         .with_entity::<Button>("New game".into())
///         .with_entity::<Button>("Settings".into())
///         .with_entity::<Button>("Exit".into());
/// }
///
/// struct Button(String);
///
/// impl EntityMainComponent for Button {
///     type Data = String;
///
///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
///         builder.with_self(Self(data))
///     }
/// }
/// ```
#[derive(Default)]
pub struct Application(MainFacade);

impl Application {
    /// Create an empty application.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the number of threads used to update the application.
    ///
    /// By default, the application only uses the main thread, and so this method returns `1`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, GroupBuilder};
    /// #
    /// let application = Application::new().with_thread_count(4);
    /// let thread_count = application.thread_count();
    /// assert_eq!(thread_count, 4);
    /// ```
    pub fn thread_count(&self) -> u32 {
        self.0.thread_count()
    }

    /// Set the number of threads used to update the application.
    ///
    /// If `count` is `0` or `1`, it means the updates are only done in main thread.<br>
    /// If this method is never called, the application only uses the main thread.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, GroupBuilder};
    /// #
    /// Application::new()
    ///     .with_group(build_main_group)
    ///     .with_thread_count(4)
    ///     .update();
    /// #
    /// # fn build_main_group(builder: &mut GroupBuilder<'_>) {}
    /// ```
    pub fn with_thread_count(mut self, count: u32) -> Self {
        self.0.set_thread_count(count);
        self
    }

    /// Create a group of entities.
    ///
    /// If this group is deleted, then all contained entities are also deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{Application, GroupBuilder, EntityMainComponent, EntityBuilder, Built, system};
    /// #
    /// Application::new()
    ///     .with_group(build_login_form)
    ///     .with_group(|b| build_version_info(b, "Version 1.2.3".into()))
    ///     .update();
    ///
    /// fn build_login_form(builder: &mut GroupBuilder<'_>) {
    ///     builder
    ///         .with_entity::<Text>("Username:".into())
    ///         .with_entity::<TextBox>(TextBoxType::Normal)
    ///         .with_entity::<Text>("Password:".into())
    ///         .with_entity::<TextBox>(TextBoxType::Password)
    ///         .with_entity::<Button>("Login".into());
    /// }
    ///
    /// fn build_version_info(builder: &mut GroupBuilder<'_>, version: String) {
    ///     builder
    ///         .with_entity::<Text>(version)
    ///         .on_update(system!(print_version));
    /// }
    ///
    /// fn print_version(text: &Text) {
    ///     if let Some(version) = text.label.strip_prefix("Version ") {
    ///         assert_eq!(version, "1.2.3");
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
    /// #
    /// # struct TextBox;
    /// #
    /// # impl EntityMainComponent for TextBox {
    /// #     type Data = TextBoxType;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>, _: Self::Data) -> Built {
    /// #         builder.with_self(Self)
    /// #     }
    /// # }
    /// #
    /// # enum TextBoxType {
    /// #     Normal,
    /// #     Password,
    /// # }
    /// #
    /// # struct Button;
    /// #
    /// # impl EntityMainComponent for Button {
    /// #     type Data = String;
    /// #
    /// #     fn build(builder: &mut EntityBuilder<'_, Self>, _: Self::Data) -> Built {
    /// #         builder.with_self(Self)
    /// #     }
    /// # }
    /// ```
    pub fn with_group(mut self, build_group_fn: impl FnOnce(&mut GroupBuilder<'_>)) -> Self {
        let group_idx = self.0.create_group();
        let mut group_builder = GroupBuilder::new(&mut self.0, group_idx);
        build_group_fn(&mut group_builder);
        self
    }

    /// Add a system.
    ///
    /// Systems are functions or closures able to iterate on entities to update their
    /// components, or to run actions like deleting an entity, creating a group, ...<br>
    /// Execution order of systems is undefined.
    ///
    /// Systems registered by this method are run each time
    /// [`Application::update`](crate::Application::update) is called, and iterates on all queried
    /// entities regardless their group and type.
    ///
    /// `system` must be defined using the [`system!`](crate::system!) macro.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     Application, system, EntityMainComponent, Built, EntityBuilder, GroupBuilder,
    /// #     EntityRunner
    /// # };
    /// #
    /// Application::new()
    ///     .with_group(build_group)
    ///     .update();
    ///
    /// fn build_group(builder: &mut GroupBuilder<'_>) {
    ///     builder
    ///         .with_entity::<Body>(10.)
    ///         .on_update(system!(update_position));
    /// }
    ///
    /// fn update_position(position: &mut Position, velocity: &Velocity) {
    ///     assert_eq!(position.x, 10.);
    ///     assert_eq!(position.y, 10.5);
    ///     position.x += velocity.x;
    ///     position.y += velocity.y;
    ///     assert_eq!(position.x, 12.);
    ///     assert_eq!(position.y, 15.5);
    /// }
    ///
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = f32;
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data, y: data + 0.5 })
    ///             .with(Velocity { x: 2., y: 5. })
    ///             .with_self(Self)
    ///     }
    /// }
    ///
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// struct Velocity {
    ///     x: f32,
    ///     y: f32,
    /// }
    /// ```
    pub fn on_update(mut self, system: SystemBuilder) -> Self {
        let system =
            SystemDetails::new(system.wrapper, system.component_types, None, system.actions);
        self.0.add_system(None, system);
        self
    }

    /// Update the application
    ///
    /// This runs all systems registered for application, groups and entities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     Application, system, EntityMainComponent, Built, EntityBuilder, GroupBuilder,
    /// #     EntityRunner
    /// # };
    /// #
    /// Application::new()
    ///     .with_group(build_group)
    ///     .update();
    ///
    /// fn build_group(builder: &mut GroupBuilder<'_>) {
    ///     builder.with_entity::<Body>(10.);
    /// }
    ///
    /// struct Body;
    ///
    /// impl EntityMainComponent for Body {
    ///     type Data = f32;
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder
    ///             .with(Position { x: data, y: data + 0.5 })
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
    ///         assert_eq!(position.x, 10.);
    ///         assert_eq!(position.y, 10.5);
    ///         position.x += velocity.x;
    ///         position.y += velocity.y;
    ///         assert_eq!(position.x, 12.);
    ///         assert_eq!(position.y, 15.5);
    ///     }
    /// }
    ///
    /// struct Position {
    ///     x: f32,
    ///     y: f32,
    /// }
    ///
    /// struct Velocity {
    ///     x: f32,
    ///     y: f32,
    /// }
    /// ```
    pub fn update(&mut self) {
        self.0.run_systems();
        self.0.apply_system_actions();
    }

    /// Run a system once.
    ///
    /// Systems are functions or closures able to iterate on entities to update their
    /// components, or to run actions like deleting an entity, creating a group, ...
    ///
    /// `system` must be defined using the [`system_once!`](crate::system_once!) macro.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use modor::{
    /// #     Application, system_once, GroupBuilder, EntityMainComponent, EntityBuilder, Built
    /// # };
    /// #
    /// let mut application = Application::new().with_group(build_entities);
    /// let mut values = Vec::new();
    /// application.run(system_once!(|number: &Number| values.push(number.0)));
    /// values.sort_unstable(); // entity order is undefined
    /// assert_eq!(values, [10, 20, 30]);
    ///
    /// fn build_entities(builder: &mut GroupBuilder<'_>) {
    ///     builder.with_entity::<Number>(10);
    ///     builder.with_entity::<Number>(20);
    ///     builder.with_entity::<Number>(30);
    /// }
    ///
    /// struct Number(u32);
    ///
    /// impl EntityMainComponent for Number {
    ///     type Data = u32;
    ///
    ///     fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
    ///         builder.with_self(Self(data))
    ///     }
    /// }
    /// ```
    pub fn run<S>(&mut self, system: SystemOnceBuilder<S>)
    where
        S: FnMut(&SystemData<'_>, SystemInfo),
    {
        self.0.run_system_once(system);
        self.0.apply_system_actions();
    }
}

#[cfg(test)]
mod application_tests {
    use super::*;
    use crate::{Built, EntityBuilder, EntityMainComponent, TypeAccess};
    use static_assertions::_core::any::TypeId;
    use std::convert::TryInto;

    assert_impl_all!(Application: Send);
    assert_not_impl_any!(Application: Clone);

    struct Number(u32);

    impl EntityMainComponent for Number {
        type Data = u32;
        fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
            builder.with_self(Self(data))
        }
    }

    #[test]
    fn set_thread_count() {
        let application = Application::new();

        let application = application.with_thread_count(4);

        assert_eq!(application.0.thread_count(), 4);
    }

    #[test]
    fn retrieve_thread_count() {
        let application = Application::new().with_thread_count(4);

        let thread_count = application.thread_count();

        assert_eq!(thread_count, 4);
    }

    #[test]
    fn create_group() {
        let application = Application::new();

        let mut application = application.with_group(|b| {
            b.with_entity::<Number>(10).with_entity::<Number>(20);
        });

        assert_eq!(application.0.create_group(), 2.try_into().unwrap());
        assert_eq!(application.0.create_entity(1.try_into().unwrap()), 2);
    }

    #[test]
    fn add_system() {
        let application = Application::new().with_group(|_| ());

        let mut application = application.on_update(SystemBuilder::new(
            |d, _| d.actions_mut().delete_group(1.try_into().unwrap()),
            vec![TypeAccess::Write(TypeId::of::<String>())],
            true,
        ));

        application.0.run_systems();
        application.0.apply_system_actions();
        assert_eq!(application.0.create_group(), 1.try_into().unwrap());
    }

    #[test]
    fn update() {
        let mut application = Application::new()
            .with_group(|_| ())
            .on_update(SystemBuilder::new(
                |d, _| d.actions_mut().delete_group(1.try_into().unwrap()),
                vec![TypeAccess::Write(TypeId::of::<String>())],
                true,
            ));

        application.update();

        assert_eq!(application.0.create_group(), 1.try_into().unwrap());
    }

    #[test]
    fn run_system_once() {
        let mut application = Application::new().with_group(|_| ());

        application.run(SystemOnceBuilder::new(|d, _| {
            d.actions_mut().delete_group(1.try_into().unwrap())
        }));

        assert_eq!(application.0.create_group(), 1.try_into().unwrap());
    }
}
