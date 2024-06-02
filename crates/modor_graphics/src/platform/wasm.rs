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

pub(crate) fn update_canvas_cursor(handle: &winit::window::Window, is_cursor_show: bool) {
    if let Some(canvas) = handle.canvas() {
        canvas
            .style()
            .set_property("cursor", if is_cursor_show { "auto" } else { "none" })
            .expect("cannot update canvas cursor property");
    }
}

pub(crate) fn surface_size(
    handle: &winit::window::Window,
    size: winit::dpi::PhysicalSize<u32>,
) -> winit::dpi::PhysicalSize<u32> {
    // If the size is not divided by the scale factor, then in case zoom is greater than 100%,
    // the canvas is recursively resized until reaching the maximum allowed size.
    let scale_factor = handle.scale_factor();
    winit::dpi::PhysicalSize::new(
        (f64::from(size.width) / scale_factor).round() as u32,
        (f64::from(size.height) / scale_factor).round() as u32,
    )
}

pub(crate) fn gpu_limits() -> wgpu::Limits {
    wgpu::Limits::downlevel_webgl2_defaults()
}
