use crate::components::button::ButtonStyleRegistry;
use crate::ButtonStyle;
use modor::{BuiltEntity, EntityBuilder};
use modor_text::TextModule;

pub fn module() -> impl BuiltEntity {
    EntityBuilder::new()
        .component(UiModule)
        .component(ButtonStyleRegistry::default())
        .component(ButtonStyle::default_style())
        .dependency::<TextModule, _, _>(modor_text::module)
}

/// The component that identifies the UI module entity created with [`modor_ui::module()`].
#[non_exhaustive]
#[derive(SingletonComponent, NoSystem)]
pub struct UiModule;
