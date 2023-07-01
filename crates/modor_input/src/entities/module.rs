use crate::{InputEventCollector, Keyboard, Mouse};
use modor::{BuiltEntity, EntityBuilder};

/// The main entity of the input module.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_input::*;
/// #
/// let app = App::new()
///      .with_entity(InputModule::build());
///
/// fn print_mouse_position(mouse: Single<'_, Mouse>) {
///     println!("Mouse position: {:?}", mouse.position());
/// }
/// ```
#[non_exhaustive]
#[derive(SingletonComponent)]
pub struct InputModule;

#[systems]
impl InputModule {
    /// Builds the module.
    pub fn build() -> impl BuiltEntity {
        info!("input module created");
        EntityBuilder::new()
            .component(Self)
            .child_entity(InputEventCollector::new())
            .child_entity(Mouse::new())
            .child_entity(Keyboard::new())
    }

    #[run_after(component(InputEventCollector))]
    fn finish() {}
}
