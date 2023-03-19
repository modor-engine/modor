// TODO: remove examples
// TODO: delete assets folder

use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, EntityBuilder};
use modor_graphics_new2::{
    Camera2D, Color, Material, Model, RenderTarget, Size, Texture, Window, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, PhysicsModule, Transform2D};
use std::time::Duration;

fn main() {
    App::new()
        .with_thread_count(4)
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::renderer())
        .with_entity(primary_render_target())
        .with_entity(secondary_render_target())
        .with_entity(resources())
        .with_entity(materials())
        .with_entity(UpdatedMaterialModel::build(Vec2::new(0.2, 0.2)))
        .with_entity(red_rectangle(Vec2::new(-0.35, 0.)))
        .with_entity(red_rectangle(Vec2::new(0.35, 0.)))
        .with_entity(blue_ellipse(Vec2::new(0., 0.1)))
        .with_entity(transparent_object(Vec2::new(0., 0.)))
        .run(modor_graphics_new2::runner);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ResourceKey {
    SmileyTexture,
    PrimaryCamera,
    SecondaryCamera,
    RedRectangleMaterial,
    BlueEllipseMaterial,
    TransparentMaterial,
    UpdatedMaterial,
}

fn primary_render_target() -> impl BuiltEntity {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct PrimaryTargetKey;

    EntityBuilder::new()
        .with(RenderTarget::new(PrimaryTargetKey))
        .with(Window::new().with_title("Primary window"))
        .with(Camera2D::new(ResourceKey::PrimaryCamera).with_target_key(PrimaryTargetKey))
}

fn secondary_render_target() -> impl BuiltEntity {
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
    struct SecondaryTargetKey;

    EntityBuilder::new()
        .with(RenderTarget::new(SecondaryTargetKey).with_background_color(Color::DARK_GREEN))
        .with(
            Window::new()
                .with_title("Secondary window")
                .with_size(Size::new(400, 300)),
        )
        .with(Camera2D::new(ResourceKey::SecondaryCamera).with_target_key(SecondaryTargetKey))
        .with(Transform2D::new().with_size(Vec2::new(-1., 1.)))
}

fn resources() -> impl BuiltEntity {
    EntityBuilder::new().with_child(Texture::from_static(
        ResourceKey::SmileyTexture,
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/smiley.png")),
    ))
}

fn materials() -> impl BuiltEntity {
    EntityBuilder::new()
        .with_child(Material::new(ResourceKey::RedRectangleMaterial).with_color(Color::MAROON))
        .with_child(Material::ellipse(ResourceKey::BlueEllipseMaterial).with_color(Color::BLUE))
        .with_child(
            Material::ellipse(ResourceKey::TransparentMaterial)
                .with_color(Color::WHITE.with_alpha(0.1))
                .with_texture(ResourceKey::SmileyTexture),
        )
}

fn red_rectangle(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.2, 0.4)),
        )
        .with(
            Model::rectangle(ResourceKey::RedRectangleMaterial)
                .with_camera_key(ResourceKey::PrimaryCamera),
        )
}

fn transparent_object(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.7, 0.7)),
        )
        .with(Dynamics2D::new().with_angular_velocity(-10.0_f32.to_radians()))
        .with(
            Model::rectangle(ResourceKey::TransparentMaterial)
                .with_camera_key(ResourceKey::PrimaryCamera)
                .with_camera_key(ResourceKey::SecondaryCamera),
        )
        .with(ZIndex2D::from(1))
}

fn blue_ellipse(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.9, 0.2)),
        )
        .with(Dynamics2D::new().with_angular_velocity(10.0_f32.to_radians()))
        .with(
            Model::rectangle(ResourceKey::BlueEllipseMaterial)
                .with_camera_key(ResourceKey::PrimaryCamera)
                .with_camera_key(ResourceKey::SecondaryCamera),
        )
        .with(ZIndex2D::from(2))
}

#[derive(Component)]
struct UpdatedMaterialModel {
    creation_time: Instant,
}

#[systems]
impl UpdatedMaterialModel {
    fn build(position: Vec2) -> impl BuiltEntity {
        EntityBuilder::new()
            .with(Self {
                creation_time: Instant::now(),
            })
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.3, 0.3)),
            )
            .with(
                Model::rectangle(ResourceKey::UpdatedMaterial)
                    .with_camera_key(ResourceKey::PrimaryCamera),
            )
            .with(Material::new(ResourceKey::UpdatedMaterial).with_color(Color::GRAY))
            .with(ZIndex2D::from(3))
    }

    #[run]
    fn update(&self, material: &mut Material) {
        if self.creation_time.elapsed() > Duration::from_secs(3) {
            material.color.a = 0.5;
        }
    }
}
