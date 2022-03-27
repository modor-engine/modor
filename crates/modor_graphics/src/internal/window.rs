use crate::backend::renderer::Renderer;
use crate::window::WindowSize;
use modor::{Built, Entity, EntityBuilder, World};
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub(crate) struct WindowInit {
    size: WindowSize,
    title: String,
    renderer: Option<Renderer>,
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
        })
    }

    pub(crate) fn take_renderer(&mut self) -> Option<Renderer> {
        self.renderer.take()
    }

    pub(crate) fn create_window(&mut self, event_loop: &EventLoop<()>) -> Window {
        let window = WindowBuilder::new()
            .with_title(self.title.clone())
            .with_inner_size(Size::Physical(PhysicalSize::new(
                self.size.width,
                self.size.height,
            )))
            .build(event_loop)
            .expect("failed to create window");
        self.renderer = Some(Renderer::new(&window));
        window
    }

    #[run]
    fn cleanup(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}
