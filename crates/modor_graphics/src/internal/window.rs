use crate::backend::renderer::Renderer;
use crate::window::WindowSize;
use crate::{GraphicsModule, Window};
use modor::{Built, Entity, EntityBuilder, Single, World};
use std::sync::{Arc, RwLock};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::Window as WinitWindow;
use winit::window::WindowBuilder;

pub(crate) struct WindowInit {
    size: WindowSize,
    title: String,
    renderer: Option<Renderer>,
    window: Option<Arc<RwLock<WinitWindow>>>,
}

#[singleton]
impl WindowInit {
    pub(crate) fn build<T>(size: WindowSize, title: T) -> impl Built<Self>
    where
        T: Into<String>,
    {
        EntityBuilder::new(Self {
            size,
            title: title.into(),
            renderer: None,
            window: None,
        })
    }

    pub(crate) fn create_window(&mut self, event_loop: &EventLoop<()>) -> Arc<RwLock<WinitWindow>> {
        let window = WindowBuilder::new()
            .with_title(self.title.clone())
            .with_inner_size(PhysicalSize::new(self.size.width, self.size.height))
            .build(event_loop)
            .expect("failed to create window");
        self.renderer = Some(Renderer::new(&window));
        let window = Arc::new(RwLock::new(window));
        self.window = Some(window.clone());
        window
    }

    #[run]
    fn consume(
        &mut self,
        entity: Entity<'_>,
        graphics: Single<'_, GraphicsModule>,
        mut world: World<'_>,
    ) {
        let window = self
            .window
            .take()
            .expect("internal error: window not initialized");
        let renderer = self
            .renderer
            .take()
            .expect("internal error: renderer not initialized");
        world.create_child_entity(graphics.entity().id(), Window::build(window, renderer));
        world.delete_entity(entity.id());
    }
}
