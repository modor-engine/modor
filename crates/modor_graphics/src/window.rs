use crate::backend::renderer::Renderer;
use crate::storages::core::CoreStorage;
use crate::window::internal::UpdateWindowAction;
use crate::{BackgroundColor, Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor::{Built, Entity, EntityBuilder, Query, Single, World};
use modor_physics::{Position, Scale, Shape};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

pub struct Window {
    window: Arc<RwLock<WinitWindow>>,
    core: CoreStorage,
}

#[singleton]
impl Window {
    pub(crate) fn build(window: Arc<RwLock<WinitWindow>>, renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self {
            window,
            core: CoreStorage::new(renderer),
        })
    }

    pub fn size(&self) -> SurfaceSize {
        let size = self.read_winit_window().inner_size();
        SurfaceSize {
            width: size.width,
            height: size.height,
        }
    }

    pub fn set_size(&mut self, size: SurfaceSize) {
        let size = PhysicalSize::new(size.width, size.height);
        self.write_winit_window().set_inner_size(size);
    }

    pub fn set_title(&mut self, title: &str) {
        self.write_winit_window().set_title(title);
    }

    #[run]
    fn update_size(&mut self) {
        self.core.set_size(self.size());
    }

    #[run_as(UpdateWindowAction)]
    fn update(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        self.core.update_instances(shapes);
    }

    #[run_after(UpdateWindowAction)]
    fn render(&mut self, background_color: Option<Single<'_, BackgroundColor>>) {
        let background_color = background_color.map_or(Color::BLACK, |c| c.color());
        self.core.render(background_color);
    }

    fn read_winit_window(&self) -> RwLockReadGuard<'_, WinitWindow> {
        self.window
            .read()
            .expect("internal error: cannot read inner window")
    }

    fn write_winit_window(&mut self) -> RwLockWriteGuard<'_, WinitWindow> {
        self.window
            .write()
            .expect("internal error: cannot write inner window")
    }
}

pub(crate) struct WindowInit {
    size: SurfaceSize,
    title: String,
    renderer: Option<Renderer>,
    window: Option<Arc<RwLock<WinitWindow>>>,
}

#[singleton]
impl WindowInit {
    pub(crate) fn build(size: SurfaceSize, title: String) -> impl Built<Self> {
        EntityBuilder::new(Self {
            size,
            title,
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
        self.renderer = Some(Renderer::for_surface(&window));
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

pub(crate) mod internal {
    use modor_physics::UpdatePhysicsAction;

    #[action(UpdatePhysicsAction)]
    pub struct UpdateWindowAction;
}
