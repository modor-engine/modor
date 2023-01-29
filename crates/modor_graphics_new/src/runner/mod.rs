use crate::runner::events::AppState;
use crate::settings::rendering::{DEFAULT_TARGET_HEIGHT, DEFAULT_TARGET_WIDTH};
use crate::targets::window::WindowTarget;
use modor::App;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub fn runner(app: App) {
    let event_loop = EventLoop::new();
    let window = Arc::new(RwLock::new(create_default_window(&event_loop)));
    let app = app.with_entity(WindowTarget::build(window.clone()));
    let mut state = AppState::new(app);
    state.init();
    event_loop.run(move |event, _, control_flow| match event {
        Event::Resumed => state.invalidate_surface(),
        Event::MainEventsCleared => read_window(&window).request_redraw(),
        Event::RedrawRequested(window_id) if window_id == read_window(&window).id() => {
            for event in state.gamepad_events() {
                state.treat_gamepad_event(event);
            }
            state.update();
        }
        Event::WindowEvent { event, window_id } => {
            if window_id == read_window(&window).id() {
                if event == WindowEvent::CloseRequested {
                    *control_flow = ControlFlow::Exit;
                }
                let event = Event::WindowEvent { event, window_id };
                state.treat_window_event(event);
            }
        }
        e @ (Event::NewEvents(_)
        | Event::DeviceEvent { .. }
        | Event::UserEvent(_)
        | Event::Suspended
        | Event::RedrawRequested(_)
        | Event::RedrawEventsCleared
        | Event::LoopDestroyed) => state.treat_window_event(e),
    });
}

fn read_window(window: &RwLock<Window>) -> RwLockReadGuard<'_, Window> {
    window
        .try_read()
        .expect("internal error: not readable window")
}

#[allow(clippy::let_and_return)]
pub(super) fn create_default_window(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
        .with_title("")
        .with_inner_size(PhysicalSize::new(
            DEFAULT_TARGET_WIDTH,
            DEFAULT_TARGET_HEIGHT,
        ))
        .build(event_loop)
        .expect("failed to create window");
    #[cfg(target_arch = "wasm32")]
    init_canvas(&window);
    window
}

#[cfg(target_arch = "wasm32")]
fn init_canvas(window: &Window) {
    use winit::platform::web::WindowExtWebSys;
    let canvas = window.canvas();
    canvas.set_id("modor");
    if !self.settings.has_visible_cursor {
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

pub(crate) mod events;
pub(crate) mod frame_rate;
pub(crate) mod inputs;
