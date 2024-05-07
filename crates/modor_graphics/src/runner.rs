use crate::gpu::GpuManager;
use crate::{gpu, platform, Window};
use modor::App;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{EventLoop, EventLoopWindowTarget};

pub fn run(app: App) {
    let event_loop = platform::event_loop();
    let mut state = State::new(app, &event_loop);
    platform::run_event_loop(event_loop, move |event, event_loop| {
        state.on_event(event, event_loop);
    });
}

struct State {
    app: App,
    is_suspended: bool,
}

impl State {
    fn new(mut app: App, event_loop: &EventLoop<()>) -> Self {
        app.root::<Window>().init(event_loop);
        app.root::<GpuManager>().is_window_target = true;
        Self {
            app,
            is_suspended: false,
        }
    }

    #[allow(clippy::wildcard_enum_match_arm)]
    fn on_event(&mut self, event: Event<()>, event_loop: &EventLoopWindowTarget<()>) {
        match event {
            Event::Suspended => self.is_suspended = true,
            Event::Resumed => self.init_surface(),
            Event::AboutToWait => self.app.root::<Window>().prepare_rendering(),
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::RedrawRequested => self.app.update(),
                WindowEvent::CloseRequested => event_loop.exit(),
                _ => (),
            },
            _ => (),
        }
    }

    fn init_surface(&mut self) {
        let instance = gpu::instance();
        let surface = self.app.root::<Window>().create_surface(&instance);
        self.app
            .root::<GpuManager>()
            .init(&instance, Some(&surface));
        self.app.root::<Window>().set_surface(surface);
    }
}
