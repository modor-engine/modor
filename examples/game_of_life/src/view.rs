use crate::GRID_WIDTH;
use modor::{entity, singleton, Built, EntityBuilder};
use modor_graphics::{Color, Mesh2D};
use modor_math::Vec2;
use modor_physics::Transform2D;

pub(crate) struct GridBackground;

#[singleton]
impl GridBackground {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(
                Transform2D::new()
                    .with_position(Vec2::ZERO)
                    .with_size(Vec2::ONE),
            )
            .with(Mesh2D::rectangle().with_color(Color::WHITE))
    }
}

pub(crate) struct AliveCell {
    x: usize,
    y: usize,
}

#[entity]
impl AliveCell {
    pub(crate) fn build(x: usize, y: usize) -> impl Built<Self> {
        EntityBuilder::new(Self { x, y })
            .with(Transform2D::new().with_size(Vec2::ONE / GRID_WIDTH as f32))
            .with(Mesh2D::rectangle().with_color(Color::BLACK).with_z(1.))
    }

    pub(crate) fn set_position(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }

    #[run]
    fn update(&mut self, transform: &mut Transform2D) {
        transform.position.x = (self.x as f32 + 0.5) / GRID_WIDTH as f32 - 0.5;
        transform.position.y = 0.5 - (self.y as f32 + 0.5) / GRID_WIDTH as f32;
    }
}
