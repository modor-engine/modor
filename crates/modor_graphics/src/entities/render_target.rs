use self::internal::RenderTarget;
use crate::backend::renderer::Renderer;
use crate::backend::targets::texture::TextureTarget;
use crate::backend::targets::window::WindowTarget;
use crate::entities::render_target::internal::{CaptureAction, PrepareCaptureAction};
use crate::{GraphicsModule, SurfaceSize, WindowSettings};
use modor::{Built, Entity, EntityBuilder, Single, World};
use modor_physics::Transform2D;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::{Window as WinitWindow, WindowBuilder};

pub(crate) const DEFAULT_CAMERA_TRANSFORM: Transform2D = Transform2D::new();

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
        })
        .inherit_from(RenderTarget::build(renderer))
    }

    pub(crate) fn set_size(&mut self, size: SurfaceSize) {
        self.size = size;
    }

    #[run]
    fn update_size(&mut self, target: &mut RenderTarget) {
        target.core.set_size(self.size());
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
        window
    }

    #[run]
    fn consume(
        &mut self,
        entity: Entity<'_>,
        graphics: Single<'_, GraphicsModule>,
        mut world: World<'_>,
    ) {
        if let Some(renderer) = self.renderer.take() {
            world.create_child_entity(graphics.entity().id(), Window::build(renderer));
            world.delete_entity(entity.id());
        }
    }
}

// coverage: on

/// A handler for capturing rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`GraphicsModule`](crate::GraphicsModule)
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
    ///
    /// If `size` has width or height equal to `0.`, then the capture size is unchanged.
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
    fn prepare(&mut self, surface: &mut RenderTarget) {
        if let Some(size) = self.updated_size.take() {
            surface.core.set_size(size);
        }
    }

    #[run_as(CaptureAction)]
    fn update(&mut self, surface: &mut RenderTarget) {
        let (width, height) = surface.core.renderer().target_size();
        self.buffer_size = SurfaceSize::new(width, height);
        self.buffer = surface.core.renderer().retrieve_buffer();
    }
}

pub(crate) mod internal {
    use crate::backend::renderer::Renderer;
    use crate::backend::textures::Image;
    use crate::entities::background::BackgroundColor;
    use crate::storages::core::{CoreStorage, ShapeComponents, TextComponents};
    use crate::storages::resources::fonts::FontKey;
    use crate::{
        Camera2D, Font, FrameRate, FrameRateLimit, InternalTextureConfig, Texture,
        DEFAULT_CAMERA_TRANSFORM,
    };
    use ab_glyph::FontVec;
    use modor::{Built, EntityBuilder, Filter, Query, Single, With};
    use modor_physics::PhysicsModule;
    use modor_physics::Transform2D;
    use winit::window::Window as WinitWindow;

    #[action]
    pub struct PrepareCaptureAction;

    #[action(PrepareCaptureAction, PhysicsModule, Camera2D)]
    pub struct PrepareRenderingAction;

    #[action(PrepareRenderingAction)]
    pub struct RenderAction;

    #[action(RenderAction)]
    pub struct CaptureAction;

    pub struct RenderTarget {
        pub(crate) core: CoreStorage,
    }

    #[singleton]
    impl RenderTarget {
        pub(crate) fn build(renderer: Renderer) -> impl Built<Self> {
            EntityBuilder::new(Self {
                core: CoreStorage::new(renderer),
            })
        }

        // coverage: off (no surface refresh with capture)
        pub(crate) fn refresh_surface(&mut self, window: &WinitWindow) {
            self.core.refresh_surface(window);
        }
        // coverage: on

        pub(crate) fn load_texture(&mut self, image: Image, config: &InternalTextureConfig) {
            self.core.load_texture(image, config);
        }

        pub(crate) fn load_font(&mut self, key: FontKey, font: FontVec) {
            self.core.load_font(key, font);
        }

        #[run_as(PrepareRenderingAction)]
        fn prepare_rendering(
            &mut self,
            shapes: Query<'_, ShapeComponents<'_>>,
            texts: Query<'_, TextComponents<'_>>,
            cameras: Query<'_, (&Transform2D, Filter<With<Camera2D>>)>,
            textures: Query<'_, &Texture>,
            fonts: Query<'_, &Font>,
        ) {
            let camera_transform = cameras
                .iter()
                .map(|(t, _)| t)
                .next()
                .unwrap_or(&DEFAULT_CAMERA_TRANSFORM);
            self.core.remove_not_found_resources(&textures, &fonts);
            self.core.update_instances(shapes, texts, camera_transform);
        }

        #[run_as(RenderAction)]
        fn render(
            &mut self,
            background_color: Single<'_, BackgroundColor>,
            frame_rate_limit: Option<Single<'_, FrameRateLimit>>,
        ) {
            let enable_vsync = matches!(frame_rate_limit.map(|f| f.get()), Some(FrameRate::VSync));
            self.core.toggle_vsync(enable_vsync);
            self.core.render(**background_color);
        }
    }
}
