use crate::platform::ThreadSafeRc;
use crate::{platform, Size};
use wgpu::Surface;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window as WindowHandle;

/// The main window where rendering is performed.
///
/// # Requirements
///
/// The window is open only if:
/// - [`App`](modor::App) is run with the graphics [`runner`](crate::runner())
///
/// The rendering is performed only if:
/// - [`App`](modor::App) is run with the graphics [`runner`](crate::runner())
/// - [`RenderTarget`](crate::RenderTarget) component is in the same entity
///
/// # Related components
///
/// - [`RenderTarget`](crate::RenderTarget)
///
/// # Entity functions creating this component
///
/// - [`window_target`](crate::window_target())
///
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics::module())
///     .with_entity(window())
///     .run(modor_graphics::runner);
/// # }
///
/// fn window() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .component(Window::default())
///         .with(|w| w.title = "My app".into())
///         .component(RenderTarget::new(TARGET))
///         .with(|t| t.background_color = Color::GREEN)
/// }
///
/// const TARGET: ResKey<RenderTarget> = ResKey::new("main");
/// ```
#[must_use]
#[allow(clippy::struct_excessive_bools)]
#[derive(SingletonComponent, Debug)]
pub struct Window {
    /// Title of the window.
    ///
    /// Default is no title.
    pub title: String,
    /// Whether the mouse cursor is shown when it is in the window.
    ///
    /// Default is `true`.
    pub is_cursor_shown: bool,
    /// Action executed when the close button of the window is pressed.
    ///
    /// If equal to [`WindowCloseBehavior::None`](WindowCloseBehavior::None), the method
    /// [`Window::is_closing_requested`](Window::is_closing_requested()) will return `true`.
    ///
    /// Default is [`WindowCloseBehavior::Exit`](WindowCloseBehavior::Exit).
    pub close_behavior: WindowCloseBehavior,
    size: Size,
    old_title: String,
    old_is_cursor_shown: bool,
    is_surface_refreshed: bool,
    is_closing_requested: bool,
    surface: Option<ThreadSafeRc<Surface>>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: String::new(),
            is_cursor_shown: true,
            close_behavior: WindowCloseBehavior::default(),
            size: Self::DEFAULT_SIZE,
            old_title: Self::DEFAULT_TITLE.into(),
            old_is_cursor_shown: true,
            is_surface_refreshed: false,
            is_closing_requested: false,
            surface: None,
        }
    }
}

#[systems]
impl Window {
    pub(crate) const DEFAULT_SIZE: Size = Size::new(800, 600);
    pub(crate) const DEFAULT_TITLE: &'static str = "";

    /// Returns the size of the window, which is also the size of the surface where the rendering
    /// is performed.
    ///
    /// Default size is 800x600.
    ///
    /// # Platform-specific
    ///
    /// - Android: default size is the size of the screen
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns whether the close button of the window has been pressed.
    pub fn is_closing_requested(&self) -> bool {
        self.is_closing_requested
    }

    pub(crate) fn refresh_surface(&mut self) {
        self.is_surface_refreshed = true;
        self.surface = None;
    }

    // on Windows, Window::set_title freezes the application if not run in main thread
    pub(crate) fn update(&mut self, handle: &WindowHandle, surface: &ThreadSafeRc<Surface>) {
        if self.surface.is_none() {
            self.surface = Some(surface.clone());
        }
        Self::on_change(&self.title, &mut self.old_title, |t| {
            handle.set_title(t);
        });
        Self::on_change(&self.is_cursor_shown, &mut self.old_is_cursor_shown, |v| {
            handle.set_cursor_visible(*v);
            platform::update_canvas_cursor(handle, self.is_cursor_shown);
        });
    }

    pub(crate) fn close_window(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        match self.close_behavior {
            WindowCloseBehavior::Exit => event_loop.exit(),
            WindowCloseBehavior::None => self.is_closing_requested = true,
        }
    }

    pub(crate) fn update_size(&mut self, width: u32, height: u32) {
        self.size = Size::new(width, height);
    }

    pub(crate) fn surface(&self) -> Option<ThreadSafeRc<Surface>> {
        self.surface.clone()
    }

    pub(crate) fn refreshed_surface(&mut self) -> Option<ThreadSafeRc<Surface>> {
        if self.is_surface_refreshed {
            self.is_surface_refreshed = false;
            self.surface.clone()
        } else {
            None
        }
    }

    fn on_change<T>(property: &T, current_property: &mut T, f: impl FnOnce(&T))
    where
        T: Clone + PartialEq,
    {
        if property != current_property {
            f(property);
            *current_property = property.clone();
        }
    }
}

/// The behavior of a window when the close button is pressed.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowCloseBehavior {
    /// The application is exited.
    #[default]
    Exit,
    /// Nothing happens.
    None,
}
