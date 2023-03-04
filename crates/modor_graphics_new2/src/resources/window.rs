use crate::data::size::NonZeroSize;
use crate::Size;
use modor::{Entity, World};
use wgpu::{Instance, Surface};
use winit::event_loop::{ControlFlow, EventLoopWindowTarget};
use winit::window::{Window as WindowHandle, WindowBuilder, WindowId};

#[must_use]
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug)]
pub struct Window {
    pub size: Size,
    pub title: String,
    pub is_cursor_shown: bool,
    pub close_behavior: WindowCloseBehavior,
    pub(crate) is_surface_invalid: bool,
    old_size: Size,
    old_title: String,
    old_is_cursor_shown: bool,
    is_destroyed: bool,
    handle: Option<WindowHandle>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            size: Size::new(800, 600),
            title: String::new(),
            is_cursor_shown: true,
            close_behavior: WindowCloseBehavior::default(),
            is_surface_invalid: false,
            old_size: Size::new(800, 600),
            old_title: String::new(),
            old_is_cursor_shown: true,
            is_destroyed: false,
            handle: None,
        }
    }
}

#[component]
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

    #[run]
    fn destroy(&mut self, entity: Entity<'_>, mut world: World<'_>) {
        if self.is_destroyed {
            world.delete_entity(entity.id());
        }
    }

    pub(crate) fn surface_size(&self) -> NonZeroSize {
        self.size.into()
    }

    pub(crate) fn handle_id(&self) -> WindowId {
        self.handle
            .as_ref()
            .expect("internal error: window handle not initialized")
            .id()
    }

    pub(crate) fn request_redraw(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        let handle = if let Some(handle) = &self.handle {
            handle
        } else {
            let handle = WindowBuilder::new()
                .with_title(&self.title)
                .with_inner_size(self.size)
                .build(event_loop)
                .expect("failed to create window");
            handle.set_cursor_visible(self.is_cursor_shown);
            handle.request_redraw();
            Self::init_canvas(&handle, self.is_cursor_shown);
            self.handle.insert(handle)
        };
        handle.request_redraw();
    }

    pub(crate) fn update(&mut self) {
        if let Some(handle) = &mut self.handle {
            Self::on_change(self.size, &mut self.old_size, |s| {
                handle.set_inner_size(*s);
            });
            Self::on_change(self.title.clone(), &mut self.old_title, |t| {
                handle.set_title(t); // on Windows, freeze if not run in main thread
            });
            Self::on_change(self.is_cursor_shown, &mut self.old_is_cursor_shown, |v| {
                handle.set_cursor_visible(*v);
            });
        }
    }

    pub(crate) fn close_window(&mut self, window_id: WindowId, control_flow: &mut ControlFlow) {
        if let Some(handle) = &self.handle {
            if handle.id() == window_id {
                match self.close_behavior {
                    WindowCloseBehavior::Exit => *control_flow = ControlFlow::Exit,
                    WindowCloseBehavior::Destroy => self.is_destroyed = true,
                }
            }
        }
    }

    pub(crate) fn update_size(&mut self, width: u32, height: u32, window_id: WindowId) {
        if let Some(handle) = &self.handle {
            if handle.id() == window_id {
                self.size = Size::new(width, height);
                self.old_size = Size::new(width, height);
            }
        }
    }

    #[allow(unsafe_code)]
    pub(crate) fn create_surface(&self, instance: &Instance) -> Option<Surface> {
        self.handle
            .as_ref()
            .map(|h| unsafe { instance.create_surface(h) })
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

    fn init_canvas(_handle: &WindowHandle, _is_cursor_show: bool) {
        // TODO: make it compatible with multiple windows
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = _handle.canvas();
            canvas.set_id("modor");
            if !_is_cursor_show {
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
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WindowCloseBehavior {
    #[default]
    Exit,
    Destroy,
}
