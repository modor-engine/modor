use crate::entities::render_target::{RenderTarget, WindowInit};
use crate::{BackgroundColor, Camera2D, Capture, Color, FrameRate, FrameRateLimit, SurfaceSize};
use modor::{BuiltEntity, EntityBuilder};
use modor_input::InputModule;
use modor_math::Vec2;
use modor_physics::PhysicsModule;

/// The main entity of the graphics module.
///
/// When this module is initialized, the following modules are also created if not existing:
/// - [`PhysicsModule`](PhysicsModule)
/// - [`InputModule`](InputModule) (if window mode)
///
/// # Examples
///
/// With window:
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// # fn no_run() {
/// let app = App::new()
///      .with_entity(GraphicsModule::build(
///          WindowSettings::default()
///              .size(SurfaceSize::new(640, 480))
///              .title("window title"),
///      ));
/// # }
///
/// fn print_window_size(window: Single<'_, Window>) {
///     println!("Window size: {:?}", window.size());
/// }
/// ```
///
/// Without window:
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// #
/// # fn no_run() {
/// let app = App::new()
///     .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(640, 480)));
/// # }
///
/// fn retrieve_capture(capture: Single<'_, Capture>) {
///     if let Some(buffer) = capture.buffer() {
///         println!("Capture buffer size: {:?}", capture.buffer());
///     } else {
///         println!("No capture yet");
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(SingletonComponent)]
pub struct GraphicsModule;

#[systems]
impl GraphicsModule {
    // coverage: off (window cannot be tested)
    /// Builds the module with a window.
    ///
    /// Window properties can be accessed using the [`Window`](crate::Window) entity.
    pub fn build(settings: WindowSettings) -> impl BuiltEntity {
        info!("graphics module created with `{settings:?}`");
        EntityBuilder::new()
            .with(Self)
            .with_child(WindowInit::from(settings))
            .with_child(Camera2D::build(Vec2::ZERO, Vec2::ONE))
            .with_child(BackgroundColor::from(Color::BLACK))
            .with_child(FrameRateLimit::from(FrameRate::VSync))
            .with_dependency::<PhysicsModule, _>(PhysicsModule::build())
            .with_dependency::<InputModule, _>(InputModule::build())
    }
    // coverage: on

    /// Builds the module without a window.
    ///
    /// Rendering result can be access using the [`Capture`](crate::Capture) entity.
    ///
    /// This mode is particularly useful for testing. You can use the
    /// [`assert_capture`](crate::testing::assert_capture) method to easily compare captures.
    pub fn build_windowless(capture_size: SurfaceSize) -> impl BuiltEntity {
        info!("graphics module created without window and with `{capture_size:?}`");
        EntityBuilder::new()
            .with(Self)
            .with_child(Capture::build(capture_size))
            .with_child(BackgroundColor::from(Color::BLACK))
            .with_dependency::<PhysicsModule, _>(PhysicsModule::build())
    }

    #[run_after(component(RenderTarget), component(Capture))]
    fn finish() {}
}

// coverage: off (window cannot be tested)
/// The settings of a window to create.
#[derive(Debug)]
pub struct WindowSettings {
    pub(crate) size: SurfaceSize,
    pub(crate) title: String,
    pub(crate) has_visible_cursor: bool,
}

impl Default for WindowSettings {
    fn default() -> Self {
        Self {
            size: SurfaceSize {
                width: 800,
                height: 600,
            },
            title: "My app".into(),
            has_visible_cursor: true,
        }
    }
}

impl WindowSettings {
    /// Defines the window size (800x600 by default).
    pub fn size(mut self, size: SurfaceSize) -> Self {
        self.size = size;
        self
    }

    /// Defines the window title (`"My app"` by default).
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = title.into();
        self
    }

    /// Defines whether the mouse cursor is visible in the window (`true` by default).
    pub fn has_visible_cursor(mut self, has_visible_cursor: bool) -> Self {
        self.has_visible_cursor = has_visible_cursor;
        self
    }
}
// coverage: on
