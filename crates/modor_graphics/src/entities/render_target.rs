use crate::backend::renderer::Renderer;
use crate::backend::targets::texture::TextureTarget;
use crate::backend::targets::window::WindowTarget;
use crate::entities::background_color::BackgroundColor;
use crate::entities::render_target::internal::{PrepareRenderingAction, RenderAction};
use crate::internal::PrepareCaptureAction;
use crate::storages::core::CoreStorage;
use crate::{Color, GraphicsModule, ShapeColor, SurfaceSize};
use modor::{Built, Entity, EntityBuilder, Query, Single, World};
use modor_physics::{Position, Scale, Shape};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

const DEFAULT_BACKGROUND_COLOR: Color = Color::BLACK;

pub(crate) struct RenderTarget {
    pub(crate) core: CoreStorage,
}

#[entity]
impl RenderTarget {
    pub(crate) fn build(renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self {
            core: CoreStorage::new(renderer),
        })
    }

    #[run_as(PrepareRenderingAction)]
    fn prepare_rendering(
        &mut self,
        shapes: Query<'_, (&ShapeColor, &Position, &Scale, Option<&Shape>)>,
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

// coverage: off (window cannot be tested)

/// The open window in which rendering occurs.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`GraphicsModule`](crate::GraphicsModule)
///
/// # Examples
///
/// See [`GraphicsModule`](crate::GraphicsModule).
pub struct Window {
    window: Arc<RwLock<WinitWindow>>,
}

#[singleton]
impl Window {
    pub(crate) fn build(window: Arc<RwLock<WinitWindow>>, renderer: Renderer) -> impl Built<Self> {
        EntityBuilder::new(Self { window }).inherit_from(RenderTarget::build(renderer))
    }

    /// Returns the size of the rendering area.
    pub fn size(&self) -> SurfaceSize {
        let size = self.read_winit_window().inner_size();
        SurfaceSize {
            width: size.width,
            height: size.height,
        }
    }

    #[run]
    fn update_size(&mut self, surface: &mut RenderTarget) {
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

// coverage: on

/// A handler for capturing rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`GraphicsModule`](crate::GraphicsModule)
/// - **Updated during**: [`UpdateCaptureBufferAction`](crate::UpdateCaptureBufferAction)
///
/// # Examples
///
/// See [`GraphicsModule`](crate::GraphicsModule).
// coverage: off (window cannot be tested)
pub struct Capture {
    buffer: Vec<u8>,
    buffer_size: SurfaceSize,
    updated_size: Option<SurfaceSize>,
}

#[singleton]
impl Capture {
    pub(crate) fn build(size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self {
            buffer_size: size,
            buffer: vec![],
            updated_size: Some(size),
        })
        .inherit_from(RenderTarget::build(Renderer::new(TextureTarget::new(
            size.width,
            size.height,
        ))))
    }

    /// Returns the capture size.
    pub fn size(&self) -> SurfaceSize {
        self.buffer_size
    }

    /// Sets the capture size.
    pub fn set_size(&mut self, size: SurfaceSize) {
        self.updated_size = Some(size);
    }

    /// Returns the capture as a 8-bit RGBA image buffer.
    pub fn buffer(&self) -> Option<&[u8]> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(&self.buffer)
        }
    }

    #[run_as(PrepareCaptureAction)]
    fn update_config(&mut self, surface: &mut RenderTarget) {
        if let Some(size) = self.updated_size.take() {
            surface.core.set_size(size);
        }
    }

    #[run_as(UpdateCaptureBufferAction)]
    fn update_buffer(&mut self, surface: &mut RenderTarget) {
        let (width, height) = surface.core.renderer().target_size();
        self.buffer_size = SurfaceSize::new(width, height);
        self.buffer = surface.core.renderer().retrieve_buffer();
    }
}

/// An action done when the graphics module has retrieved all data necessary for the rendering.
#[action(PrepareRenderingAction)]
pub struct UpdateGraphicsAction;

/// An action done when the rendering has been captured by the [`Capture`](crate::Capture) entity.
#[action(RenderAction)]
pub struct UpdateCaptureBufferAction;

pub(crate) mod internal {
    use modor_physics::UpdatePhysicsAction;

    #[action]
    pub struct PrepareCaptureAction;

    #[action(UpdatePhysicsAction, PrepareCaptureAction)]
    pub struct PrepareRenderingAction;

    #[action(PrepareRenderingAction)]
    pub struct RenderAction;
}
