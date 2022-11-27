use crate::{MemoryFontRef, PathFontRef};
use instant::Duration;
use log::LevelFilter;
use modor::{App, Built, EntityBuilder, With};
use modor_graphics::{
    testing, Alignment, Capture, Color, Font, FontRef, GraphicsModule, Mesh2D, Resource,
    ResourceState, SurfaceSize, Text2D, TextSize,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::thread;

struct Text;

#[entity]
impl Text {
    fn build(
        position: Vec2,
        size: Vec2,
        alignment: Alignment,
        text_size: TextSize,
    ) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Transform2D::new().with_position(position).with_size(size))
            .with(Mesh2D::rectangle().with_color(Color::WHITE))
            .with(
                Text2D::new(30., "AB\nCD")
                    .with_size(text_size)
                    .with_color(Color::DARK_GREEN)
                    .with_alignment(alignment)
                    .with_z(0.1),
            )
    }

    fn build_with_font(font_ref: impl FontRef) -> impl Built<Self> {
        EntityBuilder::new(Self).with(Transform2D::new()).with(
            Text2D::new(30., "Text")
                .with_font(font_ref)
                .with_size(TextSize::LineHeight(0.1)),
        )
    }
}

#[test]
fn display_text_with_auto_size_and_saturated_width() {
    test_text_rendering_with_multiple_alignments(
        Vec2::new(0.1, 0.3),
        TextSize::Auto,
        "tests/expected/text_auto_size_saturated_width.png",
    );
}

#[test]
fn display_text_with_auto_size_and_saturated_height() {
    test_text_rendering_with_multiple_alignments(
        Vec2::new(0.2, 0.1),
        TextSize::Auto,
        "tests/expected/text_auto_size_saturated_height.png",
    );
}

#[test]
fn display_text_with_line_height_and_saturating_width() {
    test_text_rendering_with_multiple_alignments(
        Vec2::new(0.1, 0.3),
        TextSize::LineHeight(0.08),
        "tests/expected/text_line_height_saturated_width.png",
    );
}

#[test]
fn display_text_with_line_height_and_saturating_height() {
    test_text_rendering_with_multiple_alignments(
        Vec2::new(0.2, 0.1),
        TextSize::LineHeight(0.075),
        "tests/expected/text_line_height_saturated_height.png",
    );
}

#[test]
fn display_text_with_update() {
    App::new()
        .with_log_level(LevelFilter::Info)
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Text::build_with_font(PathFontRef::ValidFont))
        .with_entity(Font::build(PathFontRef::ValidFont))
        .with_entity(Font::build(MemoryFontRef::ValidFont))
        .updated_until_all::<(), _>(Some(100), |t: &Font| {
            thread::sleep(Duration::from_millis(10));
            !matches!(t.state(), ResourceState::Loading)
        })
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/text_font.png")
        })
        .with_update::<(), _>(Text2D::use_default_font)
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/text_font_default.png")
        })
        .with_update::<(), _>(|t: &mut Text2D| t.set_font(MemoryFontRef::ValidFont))
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/text_font.png")
        })
        .with_update::<(), _>(|t: &mut Text2D| t.alignment = Alignment::BottomRight)
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/text_font_updated_alignment.png")
        })
        .with_update::<(), _>(|t: &mut Text2D| t.string = "I".into())
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/text_font_updated_string.png")
        });
}

fn test_text_rendering_with_multiple_alignments(size: Vec2, text_size: TextSize, path: &str) {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(Text::build(
            Vec2::new(-0.3, 0.3),
            size,
            Alignment::TopLeft,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0., 0.3),
            size,
            Alignment::Top,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0.3, 0.3),
            size,
            Alignment::TopRight,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(-0.3, 0.),
            size,
            Alignment::Left,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0., 0.),
            size,
            Alignment::Center,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0.3, 0.),
            size,
            Alignment::Right,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(-0.3, -0.3),
            size,
            Alignment::BottomLeft,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0., -0.3),
            size,
            Alignment::Bottom,
            text_size,
        ))
        .with_entity(Text::build(
            Vec2::new(0.3, -0.3),
            size,
            Alignment::BottomRight,
            text_size,
        ))
        .updated()
        .assert::<With<Capture>>(1, |e| testing::assert_capture(e, path));
}
