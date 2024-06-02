use winit::platform::web::{EventLoopExtWebSys, WindowExtWebSys};

pub(crate) const CANVAS_ID: &str = "modor";

pub(crate) fn init_canvas(handle: &winit::window::Window) {
    if let Some(canvas) = handle.canvas() {
        canvas.set_id(CANVAS_ID);
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&web_sys::Element::from(canvas)).ok())
            .expect("cannot append canvas to document body");
    }
}

pub(crate) fn run_event_loop<F>(event_loop: winit::event_loop::EventLoop<()>, event_handler: F)
where
    F: FnMut(winit::event::Event<()>, &winit::event_loop::EventLoopWindowTarget<()>) + 'static,
{
    event_loop.spawn(event_handler);
}

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}
