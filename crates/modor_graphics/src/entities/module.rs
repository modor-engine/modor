use crate::entities::render_target::WindowInit;
use crate::{Capture, SurfaceSize};
use modor::{Built, EntityBuilder};
use modor_physics::PhysicsModule;
use std::marker::PhantomData;

/// The main entity of the graphics module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Dependencies (created if not found)**: [`PhysicsModule`](modor_physics::PhysicsModule)
///
/// # Examples
///
/// With window:
/// ```rust
/// # use modor::{App, Single};
/// # use modor_graphics::{GraphicsModule, SurfaceSize, Window};
/// #
/// # fn no_run() {
/// let app = App::new()
///     .with_entity(GraphicsModule::build(SurfaceSize::new(640, 480), "window title"));
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
pub struct GraphicsModule(PhantomData<()>);

#[singleton]
impl GraphicsModule {
    /// Builds the module with a window.
    ///
    /// Window properties can be accessed using the [`Window`](crate::Window) entity.
    // coverage: off (window cannot be tested)
    pub fn build<T>(window_size: SurfaceSize, window_title: T) -> impl Built<Self>
    where
        T: Into<String>,
    {
        EntityBuilder::new(Self(PhantomData))
            .with_child(WindowInit::build(window_size, window_title.into()))
            .with_dependency(PhysicsModule::build())
    }
    // coverage: on

    /// Builds the module without a window.
    ///
    /// Rendering result can be access using the [`Capture`](crate::Capture) entity.
    ///
    /// This mode is particularly useful for testing. You can use the
    /// [`assert_capture`](crate::testing::assert_capture) method to easily compare captures.
    pub fn build_windowless(capture_size: SurfaceSize) -> impl Built<Self> {
        EntityBuilder::new(Self(PhantomData))
            .with_child(Capture::build(capture_size))
            .with_dependency(PhysicsModule::build())
    }
}
