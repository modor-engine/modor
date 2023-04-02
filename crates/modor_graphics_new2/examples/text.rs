use modor::{systems, App, BuiltEntity, Component, EntityBuilder, Query, Single};
use modor_graphics_new2::{
    Camera2D, Color, FrameRate, Material, Model, RenderTarget, Size, Texture, Window,
};
use modor_input::{InputModule, Key, Keyboard, Mouse};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, Dynamics2D, PhysicsModule, Transform2D,
};

fn main() {
    App::new()
        .with_entity(modor_graphics_new2::renderer())
        .with_entity(window())
        .with_entity(text())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new("TargetKey"))
        .with(Window::default().with_cursor_shown(false))
        .with(Camera2D::new("CameraKey").with_target_key("TargetKey"))
}

fn text() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle("MaterialKey::LeftScore"))
        .with_inherited(score_material())
}

fn score_material() -> impl BuiltEntity {
    // Equivalent of _score_material()
    TextMaterialBuilder::new("MaterialKey::LeftScore", "0")
        .with_material(|m| m.with_color(Color::BLUE))
        .with_text(|t| t)
}

fn _score_material() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Material::new("MaterialKey::LeftScore")
                .with_color(Color::BLUE)
                .with_texture("TextureKey::LeftScore"),
        )
        .with(Texture::from_size(
            "MaterialKey::LeftScore",
            Size::new(1, 1),
        ))
        .with(Text::new("0")) // linked to Texture
}

////////////////////////////////////

fn text2() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle("MaterialKey::Text"))
        .with(
            Material::new("MaterialKey::Text")
                .with_color(Color::BLUE)
                .with_texture("TextureKey::Background")
                .with_text_color(Color::BLACK)
                .with_text_texture("TextureKey::Text"), // cannot configure text texture part
        )
        .with(Texture::from_size("MaterialKey::Text", Size::new(1, 1)).with_repeated(false))
        .with(
            Text::new("0") // linked to Texture
                .with_alignment(Alignment::Left)
                .with_font_height(20.)
                .with_font("FontKey::Arial"),
        )
        .with(Font::from_path("FontKey::Arial", "arial.ttf"))
}
