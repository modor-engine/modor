// TODO: remove this file
// TODO: delete assets folder

use instant::Instant;
use modor::{entity, App, Built, EntityBuilder};
use modor_graphics_new2::{
    Camera2D, Color, GraphicsModule, Material, Model, RenderTarget, Resource, Size, Texture,
    Window, ZIndex2D,
};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};
use std::time::Duration;

fn main() {
    App::new()
        .with_thread_count(4)
        .with_entity(GraphicsModule::build())
        .with_entity(
            RenderTarget::new(ResourceKey::PrimaryRenderTarget)
                .into_entity()
                .with(Window::default().with_title("Primary window")),
        )
        .with_entity(
            RenderTarget::new(ResourceKey::SecondaryRenderTarget)
                .with_background_color(Color::DARK_GREEN)
                .into_entity()
                .with(
                    Window::default()
                        .with_title("Secondary window")
                        .with_size(Size::new(400, 300)),
                ),
        )
        .with_entity(
            Camera2D::new(ResourceKey::PrimaryCamera)
                .with_target_key(ResourceKey::PrimaryRenderTarget)
                .into_entity(),
        )
        .with_entity(
            Camera2D::new(ResourceKey::SecondaryCamera)
                .with_target_key(ResourceKey::SecondaryRenderTarget)
                .into_entity()
                .with(Transform2D::new().with_size(Vec2::new(-1., 1.))),
        )
        .with_entity(
            Texture::from_static(
                ResourceKey::SmileyTexture,
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/smiley.png")),
            )
            .into_entity(),
        )
        .with_entity(
            Material::rectangle(ResourceKey::RedRectangleMaterial)
                .with_color(Color::MAROON)
                .into_entity(),
        )
        .with_entity(
            Material::ellipse(ResourceKey::BlueEllipseMaterial)
                .with_color(Color::BLUE)
                .into_entity(),
        )
        .with_entity(
            Material::ellipse(ResourceKey::TransparentMaterial)
                .with_color(Color::WHITE.with_alpha(0.1))
                .with_texture(ResourceKey::SmileyTexture)
                .into_entity(),
        )
        .with_entity(UpdatedMaterialModel::build(Vec2::new(0.2, 0.2)))
        .with_entity(RedRectangle::build(Vec2::new(-0.35, 0.)))
        .with_entity(RedRectangle::build(Vec2::new(0.35, 0.)))
        .with_entity(BlueEllipse::build(Vec2::new(0., 0.1)))
        .with_entity(TransparentObject::build(Vec2::new(0., 0.)))
        .run(modor_graphics_new2::runner);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum ResourceKey {
    PrimaryRenderTarget,
    SecondaryRenderTarget,
    SmileyTexture,
    PrimaryCamera,
    SecondaryCamera,
    RedRectangleMaterial,
    BlueEllipseMaterial,
    TransparentMaterial,
    UpdatedMaterial,
}

struct RedRectangle;

#[entity]
impl RedRectangle {
    fn build(position: Vec2) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.2, 0.4)),
            )
            .with(
                Model::new(ResourceKey::RedRectangleMaterial)
                    .with_camera_key(ResourceKey::PrimaryCamera),
            )
    }
}

struct TransparentObject;

#[entity]
impl TransparentObject {
    fn build(position: Vec2) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.7, 0.7)),
            )
            .with(Dynamics2D::new().with_angular_velocity(-10.0_f32.to_radians()))
            .with(
                Model::new(ResourceKey::TransparentMaterial)
                    .with_camera_key(ResourceKey::PrimaryCamera)
                    .with_camera_key(ResourceKey::SecondaryCamera),
            )
            .with(ZIndex2D::from(1))
    }
}

struct BlueEllipse;

#[entity]
impl BlueEllipse {
    fn build(position: Vec2) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(position)
                    .with_size(Vec2::new(0.9, 0.2)),
            )
            .with(Dynamics2D::new().with_angular_velocity(10.0_f32.to_radians()))
            .with(
                Model::new(ResourceKey::BlueEllipseMaterial)
                    .with_camera_key(ResourceKey::PrimaryCamera)
                    .with_camera_key(ResourceKey::SecondaryCamera),
            )
            .with(ZIndex2D::from(2))
    }
}

struct UpdatedMaterialModel {
    creation_time: Instant,
}

#[entity]
impl UpdatedMaterialModel {
    fn build(position: Vec2) -> impl Built<Self> {
        EntityBuilder::new(Self {
            creation_time: Instant::now(),
        })
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.3, 0.3)),
        )
        .with(Model::new(ResourceKey::UpdatedMaterial).with_camera_key(ResourceKey::PrimaryCamera))
        .with(Material::rectangle(ResourceKey::UpdatedMaterial).with_color(Color::GRAY))
        .with(ZIndex2D::from(3))
    }

    #[run]
    fn update(&self, material: &mut Material) {
        if self.creation_time.elapsed() > Duration::from_secs(3) {
            material.color.a = 0.5;
        }
    }
}
