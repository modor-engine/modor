use std::rc;
use std::time::Duration;
use web_sys::Element;
use wgpu::Limits;
use winit::dpi::PhysicalSize;
use winit::platform::web::WindowExtWebSys;
use winit::window::Window as WindowHandle;

pub(crate) type ThreadSafeRc<T> = rc::Rc<T>;

pub(crate) const CANVAS_ID: &str = "modor";

pub(crate) fn init_canvas(handle: &WindowHandle) {
    if let Some(canvas) = handle.canvas() {
        canvas.set_id(CANVAS_ID);
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&Element::from(canvas)).ok())
            .expect("cannot append canvas to document body");
    }
}

pub(crate) fn update_canvas_cursor(handle: &WindowHandle, is_cursor_show: bool) {
    if let Some(canvas) = handle.canvas() {
        canvas
            .style()
            .set_property("cursor", if is_cursor_show { "auto" } else { "none" })
            .expect("cannot update canvas cursor property");
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn surface_size(handle: &WindowHandle, size: PhysicalSize<u32>) -> PhysicalSize<u32> {
    // If the size is not divided by the scale factor, then in case zoom is greater than 100%,
    // the canvas is recursively resized until reaching the maximum allowed size.
    let scale_factor = handle.scale_factor();
    PhysicalSize::new(
        (f64::from(size.width) / scale_factor).round() as u32,
        (f64::from(size.height) / scale_factor).round() as u32,
    )
}

pub(crate) fn gpu_limits() -> Limits {
    Limits::downlevel_webgl2_defaults()
}

pub(crate) fn sleep(_duration: Duration) {
    // sleep not supported, do nothing
}
