use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, TextureBuffer, TextureSource,
    Window, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, Transform2D};

fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::module())
        .with_entity(window_target())
        .with_entity(texture_target())
        .with_entity(texture())
        .with_entity(Camera2D::new(CameraKey::Window).with_target_key(TargetKey::Window))
        .with_entity(Camera2D::new(CameraKey::Texture).with_target_key(TargetKey::Texture))
        .with_entity(Material::new(MaterialKey::TextureTarget).with_texture_key(TextureKey::Target))
        .with_entity(Material::new(MaterialKey::Green).with_color(Color::DARK_GREEN))
        .with_entity(Material::new(MaterialKey::Red).with_color(Color::MAROON))
        .with_entity(target_rectangle())
        .with_entity(texture_object())
        .with_entity(window_object())
        .run(modor_graphics_new2::runner);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TargetKey {
    Window,
    Texture,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CameraKey {
    Window,
    Texture,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    TextureTarget,
    Green,
    Red,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TextureKey {
    Target,
}

fn window_target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::Window))
        .with(Window::default())
}

fn texture_target() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::Texture).with_background_color(Color::GRAY))
        .with(Texture::new(
            TextureKey::Target,
            TextureSource::Size(Size::new(200, 200)),
        ))
        .with(TextureBuffer::default())
        .with(DisplayTextureSize)
}

fn texture() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Texture::new(
            "Texture",
            TextureSource::Path("smiley.png".into()),
        ))
        .with(TextureBuffer::default())
        .with(DisplayTextureSize)
}

fn target_rectangle() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new())
        .with(Model::rectangle(MaterialKey::TextureTarget).with_camera_key(CameraKey::Window))
}

fn texture_object() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.1, 0.2)))
        .with(Dynamics2D::new().with_velocity(Vec2::new(0.04, 0.02)))
        .with(Model::rectangle(MaterialKey::Green).with_camera_key(CameraKey::Texture))
}

fn window_object() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::new(0.1, 0.2)))
        .with(Dynamics2D::new().with_velocity(-Vec2::new(0.04, 0.02)))
        .with(Model::rectangle(MaterialKey::Red).with_camera_key(CameraKey::Window))
        .with(ZIndex2D::from(1))
}

#[derive(Component)]
struct DisplayTextureSize;

#[systems]
impl DisplayTextureSize {
    #[run]
    fn update(buffer: &TextureBuffer) {
        println!("{}", buffer.get().iter().filter(|&&b| b != 0).count());
    }
}
