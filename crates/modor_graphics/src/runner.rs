use crate::{utils, FrameRate, FrameRateLimit, SurfaceSize, Window, WindowInit};
use instant::Instant;
use modor::App;
use modor_physics::DeltaTime;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

// coverage: off (window cannot be tested)

/// Run application update for each frame rendered in a window.
///
/// [`DeltaTime`](modor_physics::DeltaTime) is automatically updated.<br>
/// Frame rate is limited depending on [`FrameRateLimit`](crate::FrameRateLimit).
///
/// This runner must be used instead of a call to [`App::update`](modor::App::update)
/// inside a loop to ensure a correct window update.
///
/// # Panics
///
/// This will panic if [`GraphicsModule`](crate::GraphicsModule) does not exist or has been created
/// in windowless mode.
///
/// # Platform-specific
///
/// - Web: a canvas with id `modor` is automatically added to the HTML body.
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
    configure_logging();
    let event_loop = EventLoop::new();
    let mut window = None;
    app.run_for_singleton(|i: &mut WindowInit| window = Some(i.create_window(&event_loop)));
    let window = window.expect("`GraphicsModule` entity not found or created in windowless mode");
    let mut previous_update_end = Instant::now();
    event_loop.run(move |event, _, control_flow| match event {
        Event::Resumed => {
            app.run_for_singleton(|w: &mut WindowInit| w.create_renderer(&window));
            app.run_for_singleton(|w: &mut Window| w.update_renderer(&window));
        }
        Event::MainEventsCleared => window.request_redraw(),
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            let mut frame_rate = FrameRate::Unlimited;
            app.run_for_singleton(|i: &mut FrameRateLimit| frame_rate = i.get());
            app.run_for_singleton(|w: &mut Window| {
                let size = window.inner_size();
                w.set_size(SurfaceSize {
                    width: size.width,
                    height: size.height,
                });
            });
            utils::run_with_frame_rate(previous_update_end, frame_rate, || app.update());
            let update_end = Instant::now();
            app.run_for_singleton(|t: &mut DeltaTime| t.set(update_end - previous_update_end));
            previous_update_end = update_end;
        }
        Event::WindowEvent { event, window_id } if window_id == window.id() => match event {
            WindowEvent::Resized(size)
            | WindowEvent::ScaleFactorChanged {
                new_inner_size: &mut size,
                ..
            } => {
                app.run_for_singleton(|w: &mut Window| {
                    w.set_size(SurfaceSize {
                        width: size.width,
                        height: size.height,
                    });
                });
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        _ => {}
    });
}

fn configure_logging() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("cannot initialize logger");
    }
}
