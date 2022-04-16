use crate::backend::renderer::Renderer;
use crate::backend::targets::texture::TextureTarget;
use crate::backend::targets::window::WindowTarget;
use crate::background::BackgroundColor;
use crate::internal::PrepareCaptureAction;
use crate::storages::core::CoreStorage;
use crate::surface::internal::{PrepareRenderingAction, RenderAction};
use crate::{Color, GraphicsModule, ShapeColor};
use modor::{Built, Entity, EntityBuilder, Query, Single, World};
use modor_physics::{Position, Scale, Shape};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

const DEFAULT_BACKGROUND_COLOR: Color = Color::BLACK;

pub(crate) struct Surface {
    pub(crate) core: CoreStorage,
}

#[entity]
impl Surface {
    pub(crate) fn build(renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self {
            core: CoreStorage::new(renderer),
        })
    }

    #[run_as(PrepareRenderingAction)]
    fn prepare_rendering(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, Option<&Scale>, Option<&Shape>)>,
    ) {
        self.core.update_instances(shapes);
    }

    #[run_as(RenderAction)]
    fn render(&mut self, background_color: Option<Single<'_, BackgroundColor>>) {
        let background_color = background_color.map_or(DEFAULT_BACKGROUND_COLOR, |c| **c);
        self.core.render(background_color);
    }

    #[run_as(UpdateGraphicsAction)]
    fn finish_update() {}
}

pub struct Window {
    window: Arc<RwLock<WinitWindow>>,
}

#[singleton]
impl Window {
    pub(crate) fn build(window: Arc<RwLock<WinitWindow>>, renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self { window }).inherit_from(Surface::build(renderer))
    }

    pub fn size(&self) -> SurfaceSize {
        let size = self.read_winit_window().inner_size();
        SurfaceSize {
            width: size.width,
            height: size.height,
        }
    }

    #[run]
    fn update_size(&mut self, surface: &mut Surface) {
        surface.core.set_size(self.size());
    }

    fn read_winit_window(&self) -> RwLockReadGuard<'_, WinitWindow> {
        self.window
            .read()
            .expect("internal error: cannot read inner window")
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
        self.renderer = Some(Renderer::new(WindowTarget::new(&window)));
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

pub struct Capture {
    buffer: Vec<u8>,
    buffer_size: SurfaceSize,
    updated_size: Option<SurfaceSize>,
}

#[singleton]
impl Capture {
    pub fn build(size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self {
            buffer_size: size,
            buffer: vec![],
            updated_size: Some(size),
        })
        .inherit_from(Surface::build(Renderer::new(TextureTarget::new(
            size.width,
            size.height,
        ))))
    }

    pub fn size(&self) -> SurfaceSize {
        self.buffer_size
    }

    pub fn set_size(&mut self, size: SurfaceSize) {
        self.updated_size = Some(size);
    }

    pub fn buffer(&self) -> Option<&[u8]> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(&self.buffer)
        }
    }

    #[run_as(PrepareCaptureAction)]
    fn update_config(&mut self, surface: &mut Surface) {
        if let Some(size) = self.updated_size.take() {
            surface.core.set_size(size);
        }
    }

    #[run_as(UpdateCaptureBuffer)]
    fn update_buffer(&mut self, surface: &mut Surface) {
        let (width, height) = surface.core.renderer().target_size();
        self.buffer_size = SurfaceSize::new(width, height);
        self.buffer = surface.core.renderer().retrieve_buffer();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

impl SurfaceSize {
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

#[action(PrepareRenderingAction)]
pub struct UpdateGraphicsAction;

#[action(RenderAction)]
pub struct UpdateCaptureBuffer;

pub(crate) mod internal {
    use modor_physics::UpdatePhysicsAction;

    #[action]
    pub struct PrepareCaptureAction;

    #[action(UpdatePhysicsAction, PrepareCaptureAction)]
    pub struct PrepareRenderingAction;

    #[action(PrepareRenderingAction)]
    pub struct RenderAction;
}
