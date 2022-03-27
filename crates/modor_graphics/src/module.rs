use crate::internal::context::Context;
use crate::internal::window::WindowInit;
use crate::window::WindowSize;
use crate::Color;
use modor::{Built, Entity, EntityBuilder, SingleMut, World};
use modor_physics::PhysicsModule;

pub struct GraphicsModule {
    background_color: Color,
}

#[singleton]
impl GraphicsModule {
    pub fn build<T>(
        window_size: WindowSize,
        window_title: T,
        background_color: Color,
    ) -> impl Built<Self>
    where
        T: Into<String>,
    {
        EntityBuilder::new(Self { background_color })
            .with_child(WindowInit::build(window_size, window_title))
            .with_dependency(PhysicsModule::build())
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    #[run]
    fn create_context(
        entity: Entity<'_>,
        mut window_init: SingleMut<'_, WindowInit>,
        mut world: World<'_>,
    ) {
        if let Some(renderer) = window_init.take_renderer() {
            world.create_child_entity(entity.id(), Context::build(renderer))
        }
    }
}
