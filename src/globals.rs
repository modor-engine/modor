use crate::storages::core::{CoreStorage, SystemCallerType};
use crate::{EntityBuilder, EntityMainComponent, SystemRunner};
use std::any::{Any, TypeId};

/// A trait for defining a global type.
///
/// Globals are easily accessible in systems by using the
/// [`Glob`](crate::Glob)/[`GlobMut`](crate::GlobMut) parameters.
///
/// You can for example define global resources or importable modules using globals.
///
/// # Examples
///
/// ```
/// # use modor::{Global, GlobalBuilder, SystemRunner, system};
/// #
/// struct GraphicsModule;
///
/// impl Global for GraphicsModule {
///     fn build(builder: GlobalBuilder<'_>) -> GlobalBuilder<'_> {
///         builder.with_dependency(PhysicsModule)
///     }
///
///     fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
///         runner.run(system!(Self::update))
///     }
/// }
///
/// impl GraphicsModule {
///     fn update() {
///         // ...
///     }
/// }
///
/// struct PhysicsModule;
///
/// impl Global for PhysicsModule {
///     fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
///         runner.run(system!(Self::update))
///     }
/// }
///
/// impl PhysicsModule {
///     fn update() {
///         // ...
///     }
/// }
/// ```
pub trait Global: Any + Sync + Send {
    /// Defines actions run at the creation of the global.
    fn build(builder: GlobalBuilder<'_>) -> GlobalBuilder<'_> {
        builder
    }

    /// Defines systems to run during update.
    ///
    /// The systems are only run when a global of type `Self` exists.
    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
    }
}

/// A builder for defining a global.
///
/// # Examples
///
/// See [`Global`](crate::Global).
pub struct GlobalBuilder<'a> {
    pub(crate) core: &'a mut CoreStorage,
}

impl GlobalBuilder<'_> {
    /// Creates a new global of type `G` if no global of this type already exists.
    pub fn with_dependency<G>(self, global: G) -> Self
    where
        G: Global,
    {
        let global_idx = self.core.register_global::<G>();
        if !self.core.globals().exists(global_idx) {
            G::build(GlobalBuilder { core: self.core });
            if !self.core.globals().has_been_created(global_idx) {
                G::on_update(SystemRunner {
                    core: self.core,
                    caller_type: SystemCallerType::Global(TypeId::of::<G>()),
                    latest_action_idx: None,
                });
            }
            self.core.replace_or_add_global(global);
        }
        self
    }

    /// Creates a new entity with main component of type `E` and building data `data`.
    pub fn with_entity<E>(self, data: E::Data) -> Self
    where
        E: EntityMainComponent,
    {
        E::build(EntityBuilder::<_, ()>::new(self.core, None), data);
        self
    }
}

#[cfg(test)]
mod global_builder_tests {
    use crate::storages::archetypes::ArchetypeFilter;
    use crate::storages::core::CoreStorage;
    use crate::storages::systems::SystemProperties;
    use crate::{
        Built, EntityBuilder, EntityMainComponent, Global, GlobalBuilder, SystemBuilder,
        SystemRunner,
    };

    #[derive(Debug, PartialEq)]
    struct TestGlobal1(u32);

    impl Global for TestGlobal1 {}

    #[derive(Debug, PartialEq)]
    struct TestGlobal2(u32);

    impl Global for TestGlobal2 {
        fn build(builder: GlobalBuilder<'_>) -> GlobalBuilder<'_> {
            builder.with_entity::<TestEntity>(50)
        }

        fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
            runner.run(SystemBuilder {
                properties_fn: |_| SystemProperties {
                    component_types: vec![],
                    globals: vec![],
                    can_update: false,
                    archetype_filter: ArchetypeFilter::None,
                },
                wrapper: |d, _| d.globals.write::<Self>().unwrap().0 = 60,
            })
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestGlobal3(u32);

    impl Global for TestGlobal3 {}

    #[derive(Debug, PartialEq, Clone)]
    struct TestEntity(u32);

    impl EntityMainComponent for TestEntity {
        type Type = ();
        type Data = u32;

        fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built<'_> {
            builder.with_self(Self(data))
        }
    }

    assert_impl_all!(GlobalBuilder<'_>: Send, Unpin);

    #[test]
    fn build_global() {
        let mut core = CoreStorage::default();
        core.replace_or_add_global(TestGlobal1(10));
        let builder = GlobalBuilder { core: &mut core };
        builder
            .with_dependency(TestGlobal1(20))
            .with_dependency(TestGlobal2(30))
            .with_dependency(TestGlobal3(70))
            .with_entity::<TestEntity>(40);
        core.update();
        let global = Some(&TestGlobal1(10));
        assert_eq!(core.globals().read::<TestGlobal1>().as_deref(), global);
        let global = Some(&TestGlobal2(60));
        assert_eq!(core.globals().read::<TestGlobal2>().as_deref(), global);
        let global = Some(&TestGlobal3(70));
        assert_eq!(core.globals().read::<TestGlobal3>().as_deref(), global);
        let components = (&*core.components().read_components::<TestEntity>()).clone();
        let expected_components = ti_vec![ti_vec![], ti_vec![TestEntity(50), TestEntity(40)]];
        assert_eq!(components, expected_components);
    }
}
