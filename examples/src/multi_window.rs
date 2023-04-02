use modor::{App, BuiltEntity, EntityBuilder};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Window, WindowCloseBehavior,
};
use modor_math::Vec2;
use modor_physics::Transform2D;

// TODO: investigate warning
// TODO: fix black screen issue on Android
// TODO: run with WASM

pub fn main() {
    App::new()
        .with_entity(modor_graphics_new2::renderer())
        .with_entity(main_window())
        .with_entity(secondary_window())
        .run(modor_graphics_new2::runner);
}

fn main_window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::Main).with_background_color(Color::MAROON))
        .with(Window::default().with_title("Main window"))
        .with(Camera2D::new(CameraKey::Main).with_target_key(TargetKey::Main))
        .with(Material::new(MaterialKey::Main).with_color(Color::BLACK))
        .with_child(rectangle(CameraKey::Main, MaterialKey::Main))
}

fn secondary_window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey::Secondary).with_background_color(Color::GREEN))
        .with(
            Window::default()
                .with_title("Secondary window")
                .with_close_behavior(WindowCloseBehavior::None),
        )
        .with(Camera2D::new(CameraKey::Secondary).with_target_key(TargetKey::Secondary))
        .with(Material::new(MaterialKey::Secondary).with_color(Color::WHITE))
        .with_child(rectangle(CameraKey::Secondary, MaterialKey::Secondary))
}

fn rectangle(camera_key: CameraKey, material_key: MaterialKey) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::ONE * 0.5))
        .with(Model::rectangle(material_key).with_camera_key(camera_key))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TargetKey {
    Main,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CameraKey {
    Main,
    Secondary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum MaterialKey {
    Main,
    Secondary,
}
