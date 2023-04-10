use crate::components::font::{FontKey, FontRegistry, DEFAULT_FONT_FILE};
use crate::{Font, FontSource};
use modor::{BuiltEntity, EntityBuilder};
use modor_graphics_new2::GraphicsModule;

pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(TextModule)
        .with(FontRegistry::default())
        .with_child(Font::new(
            FontKey::Default,
            FontSource::File(DEFAULT_FONT_FILE),
        ))
        .with_dependency::<GraphicsModule, _, _>(modor_graphics_new2::module)
}

#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct TextModule;
