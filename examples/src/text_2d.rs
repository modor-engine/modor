use instant::Instant;
use modor::{systems, App, BuiltEntity, Component};
use modor_graphics::{
    instance_2d, window_target, Color, Default2DMaterial, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use modor_text::{text_2d, Font, Text, Text2DMaterial};
use std::time::Duration;

const FONT: ResKey<Font> = ResKey::new("main");

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(Font::from_path(FONT, "IrishGrover-Regular.ttf"))
        .with_entity(text())
        .with_entity(text_background())
        .run(modor_graphics::runner);
}

fn text() -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, "Loading", 300.)
        .updated(|t: &mut Text| t.font_key = FONT)
        .updated(|m: &mut Text2DMaterial| m.color = Color::GREEN)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.2))
        .component(LoadingText::default())
        .component(ZIndex2D::from(1))
}

fn text_background() -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::default())
        .updated(|m: &mut Default2DMaterial| m.color = Color::rgb(0.1, 0.1, 0.1))
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.2))
}

#[derive(Component)]
struct LoadingText {
    last_update: Instant,
}

impl Default for LoadingText {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
        }
    }
}

#[systems]
impl LoadingText {
    #[run]
    fn update(&mut self, text: &mut Text) {
        if self.last_update.elapsed() > Duration::from_secs(1) {
            let new_text = match text.content.matches('.').count() {
                0 => "Loading.",
                1 => "Loading..",
                2 => "Loading...",
                _ => "Loading",
            };
            text.content = new_text.into();
            self.last_update = Instant::now();
        }
    }
}
