use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, Entity, EntityBuilder, World};
use modor_graphics_new2::{Color, FrameRate, RenderTarget, Window};
use modor_physics::PhysicsModule;
use std::time::Duration;

// TODO: delete this example -> create manual test

pub fn main() {
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(modor_graphics_new2::module())
        .with_entity(FrameRate::Unlimited)
        .with_entity(window())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Window::default())
        .with(RenderTarget::new(()).with_background_color(Color::BLUE))
        .with(DynWindow::new())
}

#[derive(Component)]
struct DynWindow {
    action_instant: Instant,
    action2_instant: Instant,
    is_done: bool,
}

#[systems]
impl DynWindow {
    fn new() -> Self {
        Self {
            action_instant: Instant::now() + Duration::from_secs(2),
            action2_instant: Instant::now() + Duration::from_secs(4),
            is_done: false,
        }
    }

    #[run]
    fn update(&mut self, entity: Entity<'_>, mut world: World<'_>) {
        if self.is_done {
            return;
        }
        if self.action2_instant < Instant::now() {
            world.add_component(
                entity.id(),
                RenderTarget::new(()).with_background_color(Color::RED),
            );
            self.is_done = true;
        } else if self.action_instant < Instant::now() {
            world.delete_component::<RenderTarget>(entity.id());
        }
    }
}
