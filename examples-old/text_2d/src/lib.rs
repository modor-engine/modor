#![allow(clippy::cast_precision_loss, clippy::print_stdout, missing_docs)]

// TODO: remove old examples

use modor::{systems, App, BuiltEntity, Component, EntityBuilder, Single};
use modor_graphics::{
    Alignment, Color, GraphicsModule, Mesh2D, SurfaceSize, Text2D, TextSize, WindowSettings,
};
use modor_input::Keyboard;
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    App::new()
        .with_entity(GraphicsModule::build(
            WindowSettings::default()
                .size(SurfaceSize::new(800, 600))
                .title("Modor - text 2D"),
        ))
        .with_entity(TextBox::build())
        .run(modor_graphics::runner);
}

#[derive(Component)]
struct TextBox;

#[systems]
impl TextBox {
    fn build() -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self)
            .with(Transform2D::new().with_size(Vec2::new(0.6, 0.20) * 1.5))
            .with(Dynamics2D::new())
            .with(
                Text2D::new(30., "")
                    .with_color(Color::GREEN)
                    .with_alignment(Alignment::TopLeft)
                    .with_size(TextSize::LineHeight(0.05)),
            )
            .with(
                Mesh2D::rectangle()
                    .with_color(Color::rgb(0.1, 0.1, 0.1))
                    .with_z(-1.),
            )
    }

    #[run]
    fn update(text: &mut Text2D, keyboard: Single<'_, Keyboard>) {
        for character in keyboard.text().chars() {
            if character == '\n' || character == '\r' {
                text.string.push('\n');
            } else if character == 8 as char {
                text.string.pop();
            } else {
                text.string.push(character);
            }
        }
    }
}
