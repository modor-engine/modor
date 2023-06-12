use crate::{platform, Size};
use std::sync::Arc;
use wgpu::Surface;
use winit::event_loop::ControlFlow;
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
/// # Example
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics_new2::*;
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(modor_graphics_new2::module())
///     .with_entity(window())
///     .run(modor_graphics_new2::runner);
/// # }
///
/// fn window() -> impl BuiltEntity {
///     EntityBuilder::new()
///         .with(Window::default().with_title("My app"))
///         .with(RenderTarget::new(TargetKey).with_background_color(Color::GREEN))
/// }
///
/// #[derive(Clone, PartialEq, Eq, Debug, Hash)]
/// struct TargetKey;
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
    surface: Option<Arc<Surface>>,
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

    /// Returns the window with a given [`title`](#structfield.title).
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Returns the window with a given [`is_cursor_shown`](#structfield.is_cursor_shown).
    pub fn with_cursor_shown(mut self, is_cursor_shown: bool) -> Self {
        self.is_cursor_shown = is_cursor_shown;
        self
    }

    /// Returns the window with a given [`close_behavior`](#structfield.close_behavior).
    pub fn with_close_behavior(mut self, close_behavior: WindowCloseBehavior) -> Self {
        self.close_behavior = close_behavior;
        self
    }

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
    pub(crate) fn update(&mut self, handle: &mut WindowHandle, surface: &Arc<Surface>) {
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

    pub(crate) fn close_window(&mut self, control_flow: &mut ControlFlow) {
        match self.close_behavior {
            WindowCloseBehavior::Exit => *control_flow = ControlFlow::Exit,
            WindowCloseBehavior::None => self.is_closing_requested = true,
        }
    }

    pub(crate) fn update_size(&mut self, width: u32, height: u32) {
        self.size = Size::new(width, height);
    }

    pub(crate) fn surface(&self) -> Option<Arc<Surface>> {
        self.surface.clone()
    }

    pub(crate) fn refreshed_surface(&mut self) -> Option<Arc<Surface>> {
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
