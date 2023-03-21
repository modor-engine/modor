use instant::Instant;
use modor::{systems, App, BuiltEntity, Component, Entity, EntityBuilder, World};
use modor_graphics_new2::{Color, RenderTarget, Window};
use std::time::Duration;

pub fn main() {
    App::new()
        .with_entity(modor_graphics_new2::renderer())
        .with_entity(window())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Window::new())
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
            action_instant: Instant::now() + Duration::from_secs(5),
            action2_instant: Instant::now() + Duration::from_secs(10),
            is_done: false,
        }
    }

    #[run]
    fn update(&mut self, entity: Entity<'_>, mut world: World<'_>) {
        if self.is_done {
            return;
        }
        if self.action2_instant < Instant::now() {
            world.add_component(entity.id(), RenderTarget::new(()));
            self.is_done = true;
        } else if self.action_instant < Instant::now() {
            world.delete_component::<RenderTarget>(entity.id());
        }
    }
}
