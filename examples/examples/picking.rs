#![allow(missing_docs)]

use modor::{
    systems, App, BuiltEntity, Component, Custom, EntityBuilder, NoSystem, Query, SingleRef,
    SingletonComponent, With,
};
use modor_graphics::{
    instance_2d, instance_group_2d, texture_target, window_target, Color, Default2DMaterial, Pixel,
    Size, TextureBuffer, TextureBufferPartUpdate, ZIndex2D, TARGET_TEXTURES, TEXTURE_CAMERAS_2D,
    WINDOW_CAMERA_2D, WINDOW_TARGET,
};
use modor_input::Mouse;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_picking::{NoPicking, PickingBuffer};

// TODO: refactor this example

#[modor::modor_main]
fn main() {
    App::new()
        .with_entity(modor_picking::module())
        .with_entity(window_target())
        .with_entity(texture_target(0, Size::new(800, 600), false))
        .with_entity(object_in_texture())
        .with_entity(rectangle_instance_group())
        .with_entity(ellipse_instance_group())
        .with_entity(texture_rendering(Vec2::new(-0.25, -0.25)))
        .with_entity(instance(Vec2::new(-0.25, 0.25)).component(Rectangle))
        .with_entity(instance(Vec2::new(0.25, -0.25)).component(Rectangle))
        .with_entity(instance(Vec2::new(0.25, 0.25)).component(Ellipse))
        .with_entity(
            instance(Vec2::new(0., 0.))
                .component(Rectangle)
                .component(ZIndex2D::from(1)),
        )
        .with_entity(picking_indicator())
        .run(modor_graphics::runner);
}

fn rectangle_instance_group() -> impl BuiltEntity {
    instance_group_2d::<With<Rectangle>>(WINDOW_CAMERA_2D, Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.color = Color::BLUE)
}

fn ellipse_instance_group() -> impl BuiltEntity {
    instance_group_2d::<With<Ellipse>>(WINDOW_CAMERA_2D, Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.color = Color::GREEN)
        .updated(|m: &mut Default2DMaterial| m.is_ellipse = true)
}

fn instance(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| t.position = position)
        .with(|t| t.size = Vec2::ONE * 0.3)
}

fn texture_rendering(position: Vec2) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.texture_key = Some(TARGET_TEXTURES.get(0)))
        .updated(|t: &mut Transform2D| t.position = position)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.3)
}

fn object_in_texture() -> impl BuiltEntity {
    instance_2d(TEXTURE_CAMERAS_2D.get(0), Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.color = Color::RED)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.3, 0.2))
}

fn picking_indicator() -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
        .updated(|m: &mut Default2DMaterial| m.color = Color::WHITE.with_alpha(0.5))
        .updated(|t: &mut Transform2D| t.size = Vec2::ZERO)
        .component(ZIndex2D::from(u16::MAX))
        .component(NoPicking::default())
        .component(PickingIndicator::default())
}

#[derive(Component, NoSystem)]
struct Rectangle;

#[derive(Component, NoSystem)]
struct Ellipse;

#[derive(SingletonComponent)]
struct PickingIndicator {
    mouse_position: Pixel,
    position: Vec2,
    size: Vec2,
}

impl Default for PickingIndicator {
    fn default() -> Self {
        Self {
            mouse_position: Pixel::new(0, 0),
            position: Vec2::ZERO,
            size: Vec2::ZERO,
        }
    }
}

#[systems]
impl PickingIndicator {
    #[run_as(action(TextureBufferPartUpdate))]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn update_picking(
        &mut self,
        mut picking_buffer: Custom<PickingBuffer<'_>>,
        mouse: SingleRef<'_, '_, Mouse>,
    ) {
        let mouse = mouse.get();
        self.mouse_position = Pixel::new(mouse.position.x as u32, mouse.position.y as u32);
        picking_buffer.register(self.mouse_position, WINDOW_TARGET);
    }

    #[run_after(component(TextureBuffer), component(Transform2D))]
    fn update(
        &mut self,
        picking_buffer: Custom<PickingBuffer<'_>>,
        transforms: Query<'_, &Transform2D>,
    ) {
        self.size = Vec2::ZERO;
        if let Some(entity_id) = picking_buffer.entity_id(self.mouse_position, WINDOW_TARGET) {
            if let Some(transform) = transforms.get(entity_id) {
                self.position = transform.position;
                self.size = transform.size;
            }
        }
    }

    #[run_after_previous]
    fn update_transform(&self, transform: &mut Transform2D) {
        transform.position = self.position;
        transform.size = self.size;
    }
}
