#![allow(missing_docs)]

use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{Camera2D, Color, Material, Model, RenderTarget, Window};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_resources::ResKey;
use modor_text::{text_material, Font, Text};
use std::time::Duration;

const CAMERA: ResKey<Camera2D> = ResKey::new("main");
const FONT: ResKey<Font> = ResKey::new("main");

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window())
        .with_entity(Font::from_path(FONT, "IrishGrover-Regular.ttf"))
        .with_entity(text())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    let target_key = ResKey::unique("window");
    EntityBuilder::new()
        .component(RenderTarget::new(target_key))
        .component(Window::default())
        .component(Camera2D::new(CAMERA, target_key))
}

fn text() -> impl BuiltEntity {
    let material_key = ResKey::unique("text");
    text_material(material_key, "Loading", 300.)
        .updated(|t: &mut Text| t.font_key = FONT)
        .updated(|m: &mut Material| m.color = Color::rgb(0.1, 0.1, 0.1))
        .updated(|m: &mut Material| m.front_color = Color::WHITE)
        .component(Transform2D::new())
        .with(|t| *t.size = Vec2::new(1., 0.2))
        .component(Model::rectangle(material_key, CAMERA))
        .component(LoadingText::default())
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
