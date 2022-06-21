#![allow(missing_docs)]

use modor::{entity, App, Built, EntityBuilder};
use modor_graphics::{Camera2D, Color, GraphicsModule, ShapeColor, WindowSettings};
use modor_math::{Quat, Vec3};
use modor_physics::{Position, Rotation, Shape, Size};

struct Object;

#[entity]
impl Object {
    fn build_center() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xyz(0.5, 0.5, 1.)))
            .with(Size::from(Vec3::xy(0.1, 0.1)))
            .with(ShapeColor::from(Color::WHITE))
            .with(Shape::Circle2D)
    }

    fn build_rectangle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(0.25, 0.25)))
            .with(Size::from(Vec3::xy(0.4, 0.25)))
            .with(ShapeColor::from(Color::GREEN))
            .with(Shape::Rectangle2D)
    }

    fn build_circle() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(Position::from(Vec3::xy(-0.25, 0.25)))
            .with(Size::from(Vec3::xy(0.4, 0.25)))
            .with(ShapeColor::from(Color::BLUE))
            .with(Shape::Circle2D)
    }
}

struct CustomCamera;

#[entity]
impl CustomCamera {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(Camera2D::build_rotated(
            Position::from(Vec3::xy(0.5, 0.5)),
            Size::from(Vec3::xy(3., 3.)),
            Rotation::from(Quat::from_z(20_f32.to_radians())),
        ))
    }

    #[run]
    fn rotate(rotation: &mut Rotation) {
        **rotation = rotation.with_rotation(Quat::from_z(0.001));
    }
}

fn main() {
    input::main();
    App::new()
        .with_entity(GraphicsModule::build(WindowSettings::default()))
        .with_entity(CustomCamera::build())
        .with_entity(Object::build_center())
        .with_entity(Object::build_circle())
        .with_entity(Object::build_rectangle())
        .run(modor_graphics::runner);
}
