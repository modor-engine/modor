use crate::components::font::{FontKey, FontRegistry, DEFAULT_FONT_FILE};
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
        .with(TextModule)
        .with(FontRegistry::default())
        .with_child(Font::new(
            FontKey::Default,
            FontSource::File(DEFAULT_FONT_FILE),
        ))
        .with_dependency::<GraphicsModule, _, _>(modor_graphics::module)
}

/// The component that identifies the text module entity created with [`module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct TextModule;
