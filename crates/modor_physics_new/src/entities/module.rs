use crate::components::pipeline::Pipeline2D;
use crate::DeltaTime;
use modor::{BuiltEntity, EntityBuilder};
use std::time::Duration;

// TODO: make it graphics module dependency

/// Creates the physics module.
///
/// If this entity is not created, physics components will have no effect.
///
/// The created entity can be identified using the [`PhysicsModule`] component.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(modor_physics_new::module());
/// ```
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(DeltaTime {
            duration: Duration::ZERO,
        })
        .component(Pipeline2D::default())
}

/// The component that identifies the physics module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct PhysicsModule;
