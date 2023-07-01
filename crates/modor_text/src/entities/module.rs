use crate::components::font::{FontRegistry, DEFAULT_FONT, DEFAULT_FONT_FILE};
use crate::{Font, FontSource};
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics::GraphicsModule;

/// Creates the text module.
///
/// If this entity is not created, no text rendering will be performed.
///
/// The created entity can be identified using the [`TextModule`] component.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// #
/// App::new()
///     .with_entity(modor_text::module());
/// ```
pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(TextModule)
        .component(FontRegistry::default())
        .child_entity(Font::new(DEFAULT_FONT, FontSource::File(DEFAULT_FONT_FILE)))
        .dependency::<GraphicsModule, _, _>(modor_graphics::module)
}

/// The component that identifies the text module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct TextModule;
