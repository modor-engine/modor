use crate::{utils, FrameRate, FrameRateLimit, WindowInit};
use modor::App;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::time::Instant;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window as WinitWindow;

// coverage: off (window cannot be tested)

/// Run application update for each frame rendered in a window.
///
/// This runner must be used instead of a call to [`App::update`](modor::App::update)
/// inside a loop to ensure a correct window update.
///
/// # Panics
///
/// This will panic if [`GraphicsModule`](crate::GraphicsModule) does not exist or has been created
/// in windowless mode.
///
/// # Examples
///
/// ```rust
/// # use modor::App;
/// # use modor_graphics::{GraphicsModule, SurfaceSize};
/// #
/// # fn no_run() {
/// App::new()
///     .with_entity(GraphicsModule::build(SurfaceSize::new(640, 480), "title"))
///     .run(modor_graphics::runner);
/// # }
/// ```
#[allow(clippy::wildcard_enum_match_arm)]
pub fn runner(mut app: App) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut window = None;
    app.run_for_singleton(|i: &mut WindowInit| window = Some(i.create_window(&event_loop)));
    let window = window.expect("`GraphicsModule` entity not found or created in windowless mode");
    let mut previous_update_end = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => read_window(&window).request_redraw(),
        Event::RedrawRequested(window_id) if window_id == read_window(&window).id() => {
            let mut frame_rate = FrameRate::Unlimited;
            app.run_for_singleton(|i: &mut FrameRateLimit| frame_rate = i.get());
            utils::run_with_frame_rate(previous_update_end, frame_rate, || app.update());
            previous_update_end = Instant::now();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == read_window(&window).id() => {
            *control_flow = ControlFlow::Exit;
        }
        _ => {}
    });
}

fn read_window(window: &Arc<RwLock<WinitWindow>>) -> RwLockReadGuard<'_, WinitWindow> {
    window.read().expect("internal error: cannot read window")
}
