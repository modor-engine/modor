use crate::input::events;
use crate::input::gamepads::Gamepads;
use crate::runner::app::RunnerApp;
use crate::runner::display::Display;
use crate::{platform, Window};
use instant::Instant;
use modor::App;
use modor_input::{Fingers, Keyboard, Mouse};
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, TouchPhase, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::{Window as WindowHandle, WindowBuilder};

const MAX_FRAME_TIME: Duration = Duration::from_secs(1);

pub(super) struct RunnerState {
    app: RunnerApp,
    gamepads: Gamepads,
    previous_update_end: Instant,
    is_suspended: bool,
    window: WindowHandle,
    window_frame_time: Option<Duration>,
    display: Option<Display>,
}

impl RunnerState {
    pub(super) fn new(app: App, event_loop: &EventLoop<()>) -> Self {
        let mut app = RunnerApp::new(app);
        let window_size = Window::DEFAULT_SIZE;
        let window = WindowBuilder::new()
            .with_visible(false)
            .with_inner_size(PhysicalSize::new(window_size.width, window_size.height))
            .with_title(Window::DEFAULT_TITLE)
            .build(event_loop)
            .expect("internal error: cannot create main window");
        platform::init_canvas(&window);
        Self {
            gamepads: Gamepads::new(&mut app),
            app,
            previous_update_end: Instant::now(),
            is_suspended: false,
            window_frame_time: Self::window_frame_time(&window),
            window,
            display: None,
        }
    }

    #[allow(
        clippy::wildcard_enum_match_arm,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    pub(super) fn treat_event(&mut self, event: Event<()>, event_loop: &EventLoopWindowTarget<()>) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::Resumed => self.invalidate_surface(),
            Event::AboutToWait => self.window.request_redraw(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.app
                    .update_mouse(|m| events::update_mouse_motion(m, delta));
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => self.update(),
                WindowEvent::CloseRequested => {
                    self.app.close_window(event_loop);
                }
                WindowEvent::Resized(size) => {
                    self.app.update_window_size(PhysicalSize::new(
                        (f64::from(size.width) / self.window.scale_factor()).round() as u32,
                        (f64::from(size.height) / self.window.scale_factor()).round() as u32,
                    ));
                }
                // TODO: is it testable now ?
                // coverage: off (untestable as new_inner_size is a reference)
                WindowEvent::ScaleFactorChanged { .. } => {
                    let size = self.window.inner_size();
                    self.app.update_window_size(PhysicalSize::new(
                        (f64::from(size.width) / self.window.scale_factor()).round() as u32,
                        (f64::from(size.height) / self.window.scale_factor()).round() as u32,
                    ));
                }
                // coverage: on
                WindowEvent::MouseInput { button, state, .. } => {
                    self.app
                        .update_mouse(|m| events::update_mouse_button(m, button, state));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    self.app
                        .update_mouse(|m| events::update_mouse_wheel(m, delta));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.app
                        .update_mouse(|m| events::update_mouse_position(m, position));
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    self.app
                        .update_keyboard(|k| events::update_keyboard_key(k, &event));
                    self.app
                        .update_keyboard(|k| events::update_entered_text(k, &event));
                }
                WindowEvent::Touch(touch) => match touch.phase {
                    TouchPhase::Started => {
                        self.app.update_fingers(|f| events::press_finger(f, touch));
                    }
                    TouchPhase::Moved => {
                        self.app.update_fingers(|f| events::move_finger(f, touch));
                    }
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.app
                            .update_fingers(|f| events::release_finger(f, touch));
                    }
                },
                _ => (),
            },
            _ => (),
        }
    }

    #[allow(unused)]
    pub(super) fn run(&mut self, f: impl FnOnce(App, &mut WindowHandle) -> App) {
        self.app.run(|a| f(a, &mut self.window));
    }

    fn window_frame_time(window: &WindowHandle) -> Option<Duration> {
        window.current_monitor().and_then(|m| {
            m.video_modes()
                .map(|m| m.refresh_rate_millihertz())
                .filter_map(|r| (r > 0).then_some(r))
                .map(|r| Duration::from_secs_f64(1000. / f64::from(r)))
                .fold(None, |a, b| Some(a.map_or(b, |a: Duration| a.min(b))))
        })
    }

    fn invalidate_surface(&mut self) {
        if let Some(display) = &mut self.display {
            display.refresh_surface(&self.window);
        } else {
            self.display = Some(Display::new(&self.window));
        };
        self.app.refresh_surface();
    }

    fn update(&mut self) {
        if let Some(display) = &self.display {
            self.gamepads.treat_events(&mut self.app);
            self.app.update_gamepads(modor_input::Gamepads::sync_d_pad);
            self.app.update(&mut self.window, display);
            self.app.update_keyboard(Keyboard::refresh);
            self.app.update_mouse(Mouse::refresh);
            self.app.update_fingers(Fingers::refresh);
            self.app.update_gamepads(modor_input::Gamepads::refresh);
            self.app
                .frame_rate()
                .sleep(self.previous_update_end, self.window_frame_time);
            let update_end = Instant::now();
            self.update_delta_time(update_end);
            self.previous_update_end = update_end;
        } // coverage: off (`else` case only happens on Android)
          // coverage: on
    }

    fn update_delta_time(&mut self, update_end: Instant) {
        let delta_time = if self.is_suspended {
            self.is_suspended = false;
            Duration::ZERO
        } else {
            (update_end - self.previous_update_end).min(MAX_FRAME_TIME)
        };
        self.app.update_delta_time(delta_time);
    }
}
