use crate::data::size::NonZeroSize;
use crate::{platform, Size};
use std::sync::Arc;
use wgpu::Surface;
use winit::event_loop::ControlFlow;
use winit::window::Window as WindowHandle;

#[must_use]
#[allow(clippy::struct_excessive_bools)]
#[derive(SingletonComponent, Debug)]
pub struct Window {
    pub size: Size,
    pub title: String,
    pub is_cursor_shown: bool,
    pub close_behavior: WindowCloseBehavior,
    old_size: Size,
    old_title: String,
    old_is_cursor_shown: bool,
    is_surface_refreshed: bool,
    is_closing_requested: bool,
    surface: Option<Arc<Surface>>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            size: Size::new(800, 600),
            title: String::new(),
            is_cursor_shown: true,
            close_behavior: WindowCloseBehavior::default(),
            old_size: Size::new(1, 1),
            old_title: String::new(),
            old_is_cursor_shown: true,
            is_surface_refreshed: false,
            is_closing_requested: false,
            surface: None,
        }
    }
}

#[systems]
impl Window {
    pub fn with_size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_cursor_shown(mut self, is_cursor_shown: bool) -> Self {
        self.is_cursor_shown = is_cursor_shown;
        self
    }

    pub fn with_close_behavior(mut self, close_behavior: WindowCloseBehavior) -> Self {
        self.close_behavior = close_behavior;
        self
    }

    pub fn is_closing_requested(&self) -> bool {
        self.is_closing_requested
    }

    pub(crate) fn surface_size(&self) -> NonZeroSize {
        self.size.into()
    }

    pub(crate) fn refresh_surface(&mut self, surface: Arc<Surface>) {
        self.is_surface_refreshed = true;
        self.surface = Some(surface);
    }

    // on Windows, Window::set_title freezes the application if not run in main thread
    pub(crate) fn update(&mut self, handle: &mut WindowHandle, surface: &Arc<Surface>) {
        if self.surface.is_none() {
            self.surface = Some(surface.clone());
        }
        Self::on_change(self.size, &mut self.old_size, |s| {
            handle.set_inner_size(*s);
        });
        Self::on_change(self.title.clone(), &mut self.old_title, |t| {
            handle.set_title(t);
        });
        Self::on_change(self.is_cursor_shown, &mut self.old_is_cursor_shown, |v| {
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
        self.old_size = Size::new(width, height);
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

    fn on_change<T>(property: T, current_property: &mut T, f: impl FnOnce(&T))
    where
        T: Clone + PartialEq,
    {
        if &property != current_property {
            f(&property);
            *current_property = property;
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowCloseBehavior {
    #[default]
    Exit,
    None,
}
