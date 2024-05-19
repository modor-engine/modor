use crate::gpu::GpuManager;
use crate::{platform, Window};
use modor::log::Level;
use modor::{App, Node, RootNode, Visit};
use std::marker::PhantomData;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};
use winit::window::WindowBuilder;

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
                WindowEvent::RedrawRequested => {
                    if let Some(app) = &mut self.app {
                        app.update();
                    }
                }
                WindowEvent::CloseRequested => event_loop.exit(),
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
}

#[derive(Default, RootNode, Node, Visit)]
struct RunnerRoot;
