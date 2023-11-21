use crate::data::StyleProperty;
use modor::Custom;
use modor_resources::{ResKey, Resource, ResourceAccessor, ResourceRegistry, ResourceState};
use modor_text::Text;

pub const DEFAULT_BUTTON_STYLE: ResKey<ButtonStyle> = ResKey::new("default(modor_ui)");

pub(crate) type ButtonStyleRegistry = ResourceRegistry<ButtonStyle>;

#[derive(Component, Debug)]
pub struct ButtonStyle {
    pub font_height: StyleProperty<f32>,
    key: ResKey<Self>,
}

#[systems]
impl ButtonStyle {
    pub fn new(key: ResKey<Self>) -> Self {
        Self {
            font_height: StyleProperty::Fixed(100.),
            key,
        }
    }

    pub(crate) fn default_style() -> Self {
        Self {
            font_height: StyleProperty::Fixed(100.),
            key: DEFAULT_BUTTON_STYLE,
        }
    }
}

impl Resource for ButtonStyle {
    fn key(&self) -> ResKey<Self> {
        self.key
    }

    fn state(&self) -> ResourceState<'_> {
        ResourceState::Loaded
    }
}

#[derive(Component, Debug)]
pub struct Button {
    style_key: ResKey<ButtonStyle>,
}

#[systems]
impl Button {
    pub fn new(style_key: ResKey<ButtonStyle>) -> Self {
        Self { style_key }
    }

    #[run_after(component(ButtonStyle), component(ButtonStyleRegistry))]
    fn update(&self, text: &mut Text, style: Custom<ResourceAccessor<'_, ButtonStyle>>) {
        if let Some(style) = style.get(self.style_key) {
            if let StyleProperty::Fixed(font_height) = style.font_height {
                text.font_height = font_height;
            }
        }
    }
}
