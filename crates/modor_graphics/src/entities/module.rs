use crate::entities::render_target::WindowInit;
use crate::{Camera2D, Capture, SurfaceSize};
use modor::{Built, EntityBuilder};
use modor_input::InputModule;
use modor_math::Vec2;
use modor_physics::PhysicsModule;

/// The main entity of the graphics module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Dependencies (created if not found)**: [`PhysicsModule`](modor_physics::PhysicsModule),
///     [`InputModule`](modor_input::InputModule)
///
/// # Examples
///
/// With window:
/// ```rust
/// # use modor::{App, Single};
/// # use modor_graphics::{GraphicsModule, SurfaceSize, Window, WindowSettings};
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
/// # use modor::{App, Single};
/// # use modor_graphics::{Capture, GraphicsModule, SurfaceSize, Window};
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
pub struct GraphicsModule;

#[singleton]
impl GraphicsModule {
    // coverage: off (window cannot be tested)
    /// Builds the module with a window.
    ///
    /// Window properties can be accessed using the [`Window`](crate::Window) entity.
    pub fn build(settings: WindowSettings) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_child(WindowInit::build(settings))
            .with_child(Camera2D::build(Vec2::ZERO, Vec2::ONE))
            .with_dependency(PhysicsModule::build())
            .with_dependency(InputModule::build())
    }
    // coverage: on

    /// Builds the module without a window.
    ///
    /// Rendering result can be access using the [`Capture`](crate::Capture) entity.
    ///
    /// This mode is particularly useful for testing. You can use the
    /// [`assert_capture`](crate::testing::assert_capture) method to easily compare captures.
    pub fn build_windowless(capture_size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with_child(Capture::build(capture_size))
            .with_dependency(PhysicsModule::build())
    }
}

// coverage: off (window cannot be tested)
/// The settings of a window to create.
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
    #[must_use]
    pub fn size(mut self, size: SurfaceSize) -> Self {
        self.size = size;
        self
    }

    /// Defines the window title (`"My app"` by default).
    #[must_use]
    pub fn title<T>(mut self, title: T) -> Self
    where
        T: Into<String>,
    {
        self.title = title.into();
        self
    }

    /// Defines whether the mouse cursor is visible in the window (`true` by default).
    #[must_use]
    pub fn has_visible_cursor(mut self, has_visible_cursor: bool) -> Self {
        self.has_visible_cursor = has_visible_cursor;
        self
    }
}
// coverage: on
