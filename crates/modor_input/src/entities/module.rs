use crate::{Fingers, Gamepads, Keyboard, Mouse, VirtualKeyboard};
use modor::{BuiltEntity, EntityBuilder};

/// Creates the input module.
///
/// If this entity is not created, input singleton components will not be created.
///
/// The created entity can be identified using the [`InputModule`] component.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(modor_input::module());
/// ```
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(Keyboard::default())
        .child_component(VirtualKeyboard::default())
        .child_component(Mouse::default())
        .child_component(Fingers::default())
        .child_component(Gamepads::default())
}

/// The component that identifies the input module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct InputModule;
