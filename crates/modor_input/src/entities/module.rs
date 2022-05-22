use crate::{InputEventCollector, Keyboard, Mouse};
use modor::{Built, EntityBuilder};
use std::marker::PhantomData;

/// The main entity of the graphics module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
///
/// # Examples
///
/// ```rust
/// # use modor::{App, Single};
/// # use modor_input::{InputModule, Mouse};
/// #
/// let app = App::new()
///      .with_entity(InputModule::build());
///
/// fn print_mouse_position(mouse: Single<'_, Mouse>) {
///     println!("Mouse position: {:?}", mouse.position());
/// }
/// ```
pub struct InputModule(PhantomData<()>);

#[singleton]
impl InputModule {
    /// Builds the module.
    pub fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(PhantomData))
            .with_child(InputEventCollector::build())
            .with_child(Mouse::build())
            .with_child(Keyboard::build())
    }
}
