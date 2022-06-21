use crate::backend::renderer::Renderer;
use crate::backend::targets::texture::TextureTarget;
use crate::backend::targets::window::WindowTarget;
use crate::entities::background::BackgroundColor;
use crate::entities::render_target::internal::{PrepareRenderingAction, RenderAction};
use crate::internal::PrepareCaptureAction;
use crate::storages::core::{CameraProperties, CoreStorage, ShapeComponents};
use crate::{
    Camera2D, Color, FrameRate, FrameRateLimit, GraphicsModule, SurfaceSize, WindowSettings,
};
use modor::{Built, Entity, EntityBuilder, Query, Single, With, World};
use modor_math::{Quat, Vec3};
use modor_physics::{Position, Rotation, Size};
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
        shapes: Query<'_, ShapeComponents<'_>>,
        cameras: Query<'_, (&Position, &Size, &Rotation), With<Camera2D>>,
    ) {
        let camera = Self::extract_camera(cameras);
        self.core.update_instances(shapes, camera);
    }

    #[run_as(RenderAction)]
    fn render(
        &mut self,
        background_color: Option<Single<'_, BackgroundColor>>,
        frame_rate_limit: Option<Single<'_, FrameRateLimit>>,
    ) {
        let background_color = background_color.map_or(DEFAULT_BACKGROUND_COLOR, |c| **c);
        let enable_vsync = matches!(frame_rate_limit.map(|l| l.get()), Some(FrameRate::VSync));
        self.core.toggle_vsync(enable_vsync);
        self.core.render(background_color);
    }

    #[run_as(UpdateGraphicsAction)]
    fn finish_update() {}

    fn extract_camera(
        cameras: Query<'_, (&Position, &Size, &Rotation), With<Camera2D>>,
    ) -> CameraProperties {
        cameras.iter().next().map_or(
            CameraProperties {
                position: Position::from(Vec3::ZERO),
                size: Size::from(Vec3::ONE),
                rotation: Rotation::from(Quat::ZERO),
            },
            |(p, s, r)| CameraProperties {
                position: *p,
                size: *s,
                rotation: *r,
            },
        )
    }
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
    size: SurfaceSize,
    refreshed_renderer: Option<Renderer>,
}

#[singleton]
impl Window {
    /// Returns the size of the rendering area.
    #[must_use]
    pub fn size(&self) -> SurfaceSize {
        self.size
    }

    pub(crate) fn build(renderer: Renderer) -> impl Built<Self> {
        let (width, height) = renderer.target_size();
        EntityBuilder::new(Self {
            size: SurfaceSize { width, height },
            refreshed_renderer: None,
        })
        .inherit_from(RenderTarget::build(renderer))
    }

    pub(crate) fn set_size(&mut self, size: SurfaceSize) {
        self.size = size;
    }

    pub(crate) fn update_renderer(&mut self, window: &WinitWindow) {
        self.refreshed_renderer = Some(Renderer::new(WindowTarget::new(window)));
    }

    #[run]
    fn update_size(&mut self, surface: &mut RenderTarget) {
        if let Some(renderer) = self.refreshed_renderer.take() {
            surface.core = CoreStorage::new(renderer);
        } else {
            surface.core.set_size(self.size());
        }
    }
}

pub(crate) struct WindowInit {
    settings: WindowSettings,
    renderer: Option<Renderer>,
}

#[singleton]
impl WindowInit {
    pub(crate) fn build(settings: WindowSettings) -> impl Built<Self> {
        EntityBuilder::new(Self {
            settings,
            renderer: None,
        })
    }

    pub(crate) fn create_renderer(&mut self, window: &WinitWindow) {
        self.renderer = Some(Renderer::new(WindowTarget::new(window)));
    }

    #[allow(clippy::let_and_return)]
    pub(crate) fn create_window(&mut self, event_loop: &EventLoop<()>) -> WinitWindow {
        let window = WindowBuilder::new()
            .with_title(self.settings.title.clone())
            .with_inner_size(PhysicalSize::new(
                self.settings.size.width,
                self.settings.size.height,
            ))
            .build(event_loop)
            .expect("failed to create window");
        window.set_cursor_visible(self.settings.has_visible_cursor);
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = window.canvas();
            canvas.set_id("modor");
            if !self.settings.has_visible_cursor {
                canvas
                    .style()
                    .set_property("cursor", "none")
                    .expect("cannot setup canvas");
            }
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
                .expect("cannot append canvas to document body");
        }
        #[cfg(not(target_os = "android"))]
        {
            self.renderer = Some(Renderer::new(WindowTarget::new(&window)));
        }
        window
    }

    #[run]
    fn consume(
        &mut self,
        entity: Entity<'_>,
        graphics: Single<'_, GraphicsModule>,
        mut world: World<'_>,
    ) {
        let renderer = if let Some(renderer) = self.renderer.take() {
            renderer
        } else {
            Renderer::new(TextureTarget::new(
                self.settings.size.width,
                self.settings.size.height,
            ))
        };
        world.create_child_entity(graphics.entity().id(), Window::build(renderer));
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
    /// Returns the capture size.
    #[must_use]
    pub fn size(&self) -> SurfaceSize {
        self.buffer_size
    }

    /// Sets the capture size.
    pub fn set_size(&mut self, size: SurfaceSize) {
        self.updated_size = Some(size);
    }

    /// Returns the capture as a 8-bit RGBA image buffer.
    #[must_use]
    pub fn buffer(&self) -> Option<&[u8]> {
        if self.buffer.is_empty() {
            None
        } else {
            Some(&self.buffer)
        }
    }

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
    use crate::UpdateCamera2DAction;
    use modor_input::UpdateInputAction;
    use modor_physics::UpdatePhysicsAction;

    #[action]
    pub struct PrepareCaptureAction;

    #[action(
        UpdatePhysicsAction,
        UpdateInputAction,
        UpdateCamera2DAction,
        PrepareCaptureAction
    )]
    pub struct PrepareRenderingAction;

    #[action(PrepareRenderingAction)]
    pub struct RenderAction;
}
