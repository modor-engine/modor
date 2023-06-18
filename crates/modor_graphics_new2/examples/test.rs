use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics_new2::{Camera2D, RenderTarget, Size, Texture, Window};

fn main() {
    App::new()
        .with_entity(modor_graphics_new2::module())
        .with_entity(target())
        .with_entity(Texture::from_buffer("TextureKey", Size::ZERO, vec![]))
        .run(modor_graphics_new2::runner);
}

fn target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Window::default())
        .with(RenderTarget::new(TargetKey))
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;
