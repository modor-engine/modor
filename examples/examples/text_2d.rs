#![allow(missing_docs)]

use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics::{Camera2D, Color, Model, RenderTarget, Window};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{Font, Text, TextMaterialBuilder};
use std::time::Duration;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window())
        .with_entity(Font::from_path(FontKey, "IrishGrover-Regular.ttf"))
        .with_entity(text())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default())
        .with(Camera2D::new(CameraKey, TargetKey))
}

fn text() -> impl BuiltEntity {
    TextMaterialBuilder::new(MaterialKey, "Loading", 300.)
        .with_text(|t| t.with_font(FontKey))
        .with_material(|t| {
            t.with_color(Color::rgb(0.1, 0.1, 0.1))
                .with_front_color(Color::WHITE)
        })
        .build()
        .with(Transform2D::new().with_size(Vec2::new(1., 0.2)))
        .with(Model::rectangle(MaterialKey, CameraKey))
        .with(LoadingText::default())
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct MaterialKey;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct FontKey;
