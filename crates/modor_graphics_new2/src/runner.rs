use crate::input::events;
use crate::input::gamepads::Gamepads;
use crate::{FrameRate, GpuContext, Renderer, Window};
use instant::Instant;
use modor::App;
use modor_input::{InputEvent, InputEventCollector};
use modor_physics::DeltaTime;
use std::mem;
use std::sync::Arc;
use std::time::Duration;
use wgpu::{Instance, Surface};
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, Event, TouchPhase, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use winit::window::{Window as WindowHandle, WindowBuilder};

const MAX_FRAME_TIME: Duration = Duration::from_secs(1);

#[allow(dead_code)]
const CANVAS_ID: &str = "modor";

pub fn runner(app: App) {
    let event_loop = EventLoop::new();
    let mut state = RunnerState::new(app, &event_loop);
    event_loop.run(move |event, _event_loop, control_flow| {
        state.treat_event(event, control_flow);
    });
}

#[doc(hidden)]
pub fn test_runner(
    app: App,
    context: &mut TestRunnerContext,
    update_count: u32,
    mut f: impl FnMut(App, &mut WindowHandle, u32) -> App,
) {
    // TODO: use cfg aliases ?
    #[cfg(any(all(unix, not(apple), not(android_platform)), target_os = "windows"))]
    {
        use winit::platform::run_return::EventLoopExtRunReturn;

        let event_loop = context
            .event_loop
            .as_mut()
            .expect("internal error: test event loop not initialized");
        let mut state = RunnerState::new(app, event_loop);
        let mut update_id = 0;
        event_loop.run_return(move |event, _event_loop, control_flow| {
            let is_update = matches!(event, Event::MainEventsCleared);
            state.treat_event(event, control_flow);
            if is_update {
                state.app.app = f(
                    mem::take(&mut state.app.app),
                    &mut state.main_window,
                    update_id,
                );
                update_id += 1;
            }
            if update_count == update_id {
                *control_flow = ControlFlow::Exit;
            }
        });
    }
    #[cfg(not(any(all(unix, not(apple), not(android_platform)), target_os = "windows")))]
    {
        panic!("test runner not supported on this platform");
    }
}

// should be created only once
#[doc(hidden)]
pub struct TestRunnerContext {
    event_loop: Option<EventLoop<()>>,
}

impl Default for TestRunnerContext {
    fn default() -> Self {
        #[cfg(any(all(unix, not(apple), not(android_platform)), target_os = "windows"))]
        {
            #[cfg(all(unix, not(apple), not(android_platform)))]
            use winit::platform::unix::EventLoopBuilderExtUnix;
            #[cfg(target_os = "windows")]
            use winit::platform::windows::EventLoopBuilderExtWindows;
            Self {
                event_loop: Some(EventLoopBuilder::new().with_any_thread(true).build()),
            }
        }
        #[cfg(not(any(all(unix, not(apple), not(android_platform)), target_os = "windows")))]
        {
            Self { event_loop: None }
        }
    }
}

struct RunnerState {
    app: RunnerApp,
    gamepads: Gamepads,
    previous_update_end: Instant,
    is_suspended: bool,
    main_window: WindowHandle,
    window_frame_time: Option<Duration>,
    display: Option<Display>,
}

impl RunnerState {
    fn new(app: App, event_loop: &EventLoop<()>) -> Self {
        let mut app = RunnerApp::new(app);
        let main_window = WindowBuilder::new()
            .with_visible(false)
            .with_inner_size(PhysicalSize::new(800, 600))
            .with_title("")
            .build(event_loop)
            .expect("internal error: cannot create main window");
        Self::init_canvas(&main_window);
        Self {
            gamepads: Gamepads::new(&mut app),
            app,
            previous_update_end: Instant::now(),
            is_suspended: false,
            window_frame_time: Self::window_frame_time(&main_window),
            main_window,
            display: None,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn treat_event(&mut self, event: Event<'_, ()>, control_flow: &mut ControlFlow) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::Resumed => self.invalidate_surface(),
            Event::MainEventsCleared => self.update(),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => {
                self.app.send_event(events::mouse_motion(delta));
            }
            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => {
                    if self.main_window.id() == window_id {
                        self.app.close_main_window(&self.main_window, control_flow);
                    }
                }
                WindowEvent::Resized(size)
                | WindowEvent::ScaleFactorChanged {
                    new_inner_size: &mut size,
                    ..
                } => {
                    if self.main_window.id() == window_id {
                        self.app.update_main_window_size(&self.main_window, size);
                    }
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    self.app.send_event(events::mouse_button(button, state));
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    self.app.send_event(events::mouse_wheel(delta));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    self.app.send_event(events::mouse_position(position));
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(event) = events::keyboard_key(input) {
                        self.app.send_event(event);
                    }
                }
                WindowEvent::ReceivedCharacter(character) => {
                    self.app.send_event(events::character(character));
                }
                WindowEvent::Touch(touch) => match touch.phase {
                    TouchPhase::Started => {
                        for event in events::started_touch(touch) {
                            self.app.send_event(event);
                        }
                    }
                    TouchPhase::Moved => self.app.send_event(events::moved_touch(touch)),
                    TouchPhase::Ended | TouchPhase::Cancelled => {
                        self.app.send_event(events::ended_touch(touch));
                    }
                },
                _ => (),
            },
            _ => (),
        }
    }

    fn window_frame_time(main_window: &WindowHandle) -> Option<Duration> {
        main_window.current_monitor().and_then(|m| {
            m.video_modes()
                .map(|m| m.refresh_rate_millihertz())
                .map(|r| Duration::from_secs_f64(1000. / f64::from(r)))
                .fold(None, |a, b| Some(a.map_or(b, |a: Duration| a.min(b))))
        })
    }

    fn invalidate_surface(&mut self) {
        let main_surface = if let Some(display) = &mut self.display {
            display.refresh_surface(&self.main_window);
            &display.main_surface
        } else {
            let display = self.display.insert(Display::new(&self.main_window));
            &display.main_surface
        };
        self.app.refresh_main_surface(main_surface);
    }

    fn update(&mut self) {
        if self.display.is_none() {
            return;
        }
        self.gamepads.treat_events(&mut self.app);
        self.main_window.request_redraw();
        self.app.update(&mut self.main_window, &self.display);
        self.app
            .frame_rate()
            .sleep(self.previous_update_end, self.window_frame_time);
        let update_end = Instant::now();
        self.update_delta_time(update_end);
        self.previous_update_end = update_end;
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

    fn init_canvas(_handle: &WindowHandle) {
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;
            let canvas = _handle.canvas();
            canvas.set_id(CANVAS_ID);
            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| doc.body())
                .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
                .expect("cannot append canvas to document body");
        }
    }
}

pub(crate) struct RunnerApp {
    app: App,
}

impl RunnerApp {
    fn new(app: App) -> Self {
        Self { app }
    }

    fn update(&mut self, main_window: &mut WindowHandle, display: &Option<Display>) {
        if let Some(display) = display {
            self.app.update_components(|r: &mut Renderer| {
                r.update(&display.renderer);
            });
            let mut is_window_found = false;
            self.app.update_components(|w: &mut Window| {
                w.update(main_window, &display.main_surface);
                is_window_found = true;
            });
            if !is_window_found {
                main_window.set_visible(false);
                main_window.set_inner_size(PhysicalSize::new(1, 1));
                main_window.set_title("");
            }
        }
        self.app.update();
    }

    fn frame_rate(&mut self) -> FrameRate {
        let mut frame_rate = FrameRate::default();
        self.app.update_components(|r: &mut FrameRate| {
            frame_rate = *r;
        });
        frame_rate
    }

    fn update_delta_time(&mut self, delta_time: Duration) {
        self.app.update_components(|t: &mut DeltaTime| {
            t.set(delta_time);
        });
    }

    fn update_main_window_size(&mut self, handle: &WindowHandle, size: PhysicalSize<u32>) {
        self.app.update_components(|w: &mut Window| {
            let PhysicalSize { width, height } = size;
            w.update_size(width, height, handle);
        });
    }

    fn close_main_window(&mut self, handle: &WindowHandle, control_flow: &mut ControlFlow) {
        let mut is_window_found = false;
        self.app.update_components(|w: &mut Window| {
            w.close_window(control_flow, handle);
            is_window_found = true;
        });
        if !is_window_found {
            *control_flow = ControlFlow::Exit;
        }
    }

    fn refresh_main_surface(&mut self, surface: &Arc<Surface>) {
        self.app.update_components(|w: &mut Window| {
            w.refresh_surface(surface.clone());
        });
    }

    pub(crate) fn send_event(&mut self, event: InputEvent) {
        self.app.update_components(|e: &mut InputEventCollector| {
            e.push(event.clone());
        });
    }
}

struct Display {
    instance: Instance,
    renderer: Arc<GpuContext>,
    main_surface: Arc<Surface>,
}

impl Display {
    fn new(main_window: &WindowHandle) -> Self {
        let instance = GpuContext::instance();
        let main_surface = Arc::new(Self::create_surface(main_window, &instance));
        Self {
            renderer: Arc::new(GpuContext::new(&instance, Some(&main_surface))),
            instance,
            main_surface,
        }
    }

    fn refresh_surface(&mut self, main_window: &WindowHandle) {
        self.main_surface = Arc::new(Self::create_surface(main_window, &self.instance));
    }

    #[allow(unsafe_code)]
    fn create_surface(handle: &WindowHandle, instance: &Instance) -> Surface {
        unsafe { instance.create_surface(handle) }
    }
}
