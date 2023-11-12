use crate::runner::display::Display;
use crate::{FrameRate, Renderer, Window};
use modor::App;
use modor_input::{Fingers, Gamepads, Keyboard, Mouse};
use modor_physics::DeltaTime;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::Window as WindowHandle;

pub(crate) struct RunnerApp {
    app: App,
    is_window_found: bool,
}

impl RunnerApp {
    pub(super) fn new(app: App) -> Self {
        Self {
            app,
            is_window_found: false,
        }
    }

    pub(super) fn update_window(&mut self, window: &mut WindowHandle, display: &Display) {
        self.app.update_components(|r: &mut Renderer| {
            r.update(&display.renderer);
        });
        let mut is_window_found = false;
        self.app.update_components(|w: &mut Window| {
            w.update(window, &display.surface);
            is_window_found = true;
        });
        if is_window_found != self.is_window_found {
            if is_window_found {
                window.set_visible(true);
            } else {
                let size = Window::DEFAULT_SIZE;
                let _ = window.request_inner_size(PhysicalSize::new(size.width, size.height));
                window.set_visible(false);
                window.set_title("");
            }
            self.is_window_found = is_window_found;
        }
    }

    pub(super) fn update(&mut self) {
        self.app.update();
    }

    #[allow(unused)]
    pub(super) fn run(&mut self, f: impl FnOnce(App) -> App) {
        self.app = f(std::mem::take(&mut self.app));
    }

    pub(super) fn frame_rate(&mut self) -> FrameRate {
        let mut frame_rate = FrameRate::default();
        self.app.update_components(|r: &mut FrameRate| {
            frame_rate = *r;
        });
        frame_rate
    }

    pub(super) fn update_delta_time(&mut self, delta_time: Duration) {
        self.app.update_components(|t: &mut DeltaTime| {
            t.set(delta_time);
        });
    }

    pub(super) fn update_window_size(&mut self, size: PhysicalSize<u32>) {
        self.app.update_components(|w: &mut Window| {
            let PhysicalSize { width, height } = size;
            w.update_size(width, height);
        });
    }

    pub(super) fn close_window(&mut self, event_loop: &EventLoopWindowTarget<()>) {
        self.app.update_components(|w: &mut Window| {
            w.close_window(event_loop);
        });
    }

    pub(super) fn refresh_surface(&mut self) {
        self.app.update_components(|w: &mut Window| {
            w.refresh_surface();
        });
    }

    pub(crate) fn update_keyboard(&mut self, update_fn: impl FnMut(&mut Keyboard)) {
        self.app.update_components(update_fn);
    }

    pub(crate) fn update_mouse(&mut self, update_fn: impl FnMut(&mut Mouse)) {
        self.app.update_components(update_fn);
    }

    pub(crate) fn update_fingers(&mut self, update_fn: impl FnMut(&mut Fingers)) {
        self.app.update_components(update_fn);
    }

    pub(crate) fn update_gamepads(&mut self, update_fn: impl FnMut(&mut Gamepads)) {
        self.app.update_components(update_fn);
    }
}
