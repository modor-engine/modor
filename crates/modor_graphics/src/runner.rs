use crate::gpu::GpuManager;
use crate::{platform, Size, Window};
use instant::Instant;
use modor::log::Level;
use modor::{App, Node, RootNode, Visit};
use modor_physics::Delta;
use std::marker::PhantomData;
use std::time::Duration;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::WindowBuilder;

const MAX_FRAME_TIME: Duration = Duration::from_secs(1);

// coverage: off (window and inputs cannot be tested)

/// Runs the application with a window.
///
/// This function also has the following effects:
/// - Inputs of the [`modor_input`] crate are updated based on window events.
/// - [`Delta`](Delta) is updated based on execution time of the last frame.
///
/// If [`App::update`](App::update) is manually used instead of this function, then no window is
/// created.
///
/// # Platform-specific
///
/// - Web: a canvas with id `modor` is automatically added to the HTML body.
/// - Android: gamepad inputs are not supported.
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_resources::*;
/// # use log::Level;
/// #
/// # fn no_run() {
/// fn main() {
///     modor_graphics::run::<Root>(Level::Info);
/// }
///
/// #[derive(Default, RootNode, Node, Visit)]
/// struct Root;
/// # }
/// ```
pub fn run<T>(level: Level)
where
    T: RootNode,
{
    let event_loop = platform::event_loop();
    let mut state = State::<T>::new(level, &event_loop);
    platform::run_event_loop(event_loop, move |event, event_loop| {
        state.on_event(event, event_loop);
    });
}

struct State<T> {
    app: Option<App>,
    window: Option<winit::window::Window>,
    level: Level,
    is_suspended: bool,
    previous_update_end: Instant,
    phantom_data: PhantomData<fn(T)>,
}

impl<T> State<T>
where
    T: RootNode,
{
    fn new(level: Level, event_loop: &EventLoop<()>) -> Self {
        Self {
            app: None,
            window: Some(Self::create_window(event_loop)),
            level,
            is_suspended: false,
            previous_update_end: Instant::now(),
            phantom_data: PhantomData,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn on_event(&mut self, event: Event<()>, event_loop: &EventLoopWindowTarget<()>) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::Resumed => self.init_surface(),
            Event::AboutToWait => {
                if let Some(app) = &mut self.app {
                    app.get_mut::<Window>().prepare_rendering();
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => self.update_app(),
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    if let Some(app) = &mut self.app {
                        app.get_mut::<Window>().size = Size::new(size.width, size.height);
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub(crate) fn create_window(event_loop: &EventLoop<()>) -> winit::window::Window {
        let size = Window::DEFAULT_SIZE;
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(size.width, size.height))
            .build(event_loop)
            .expect("internal error: cannot create main window");
        platform::init_canvas(&window);
        window
    }

    fn init_surface(&mut self) {
        if let Some(window) = self.window.take() {
            let app = self
                .app
                .get_or_insert_with(|| App::new::<RunnerRoot>(self.level));
            let instance = app.get_mut::<GpuManager>().instance.clone();
            let surface = app
                .get_mut::<Window>()
                .create_surface(&instance, Some(window));
            app.get_mut::<GpuManager>().configure_window(&surface);
            app.get_mut::<Window>().set_surface(surface);
            app.get_mut::<T>();
        } else {
            let app = self.app.as_mut().expect("internal error: not created app");
            let instance = app.get_mut::<GpuManager>().instance.clone();
            let surface = app.get_mut::<Window>().create_surface(&instance, None);
            app.get_mut::<Window>().set_surface(surface);
        }
    }

    fn update_app(&mut self) {
        if let Some(app) = &mut self.app {
            app.update();
            let update_end = Instant::now();
            app.get_mut::<Delta>().duration = if self.is_suspended {
                self.is_suspended = false;
                Duration::ZERO
            } else {
                (update_end - self.previous_update_end).min(MAX_FRAME_TIME)
            };
            self.previous_update_end = update_end;
        }
    }
}

#[derive(Default, RootNode, Node, Visit)]
struct RunnerRoot;
