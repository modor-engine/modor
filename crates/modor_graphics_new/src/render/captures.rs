use crate::rendering::Rendering;
use crate::targets::texture::TextureTarget;
use crate::targets::GpuDevice;

#[derive(Default)]
pub struct Capture {
    buffer: Vec<u8>,
}

#[component]
impl Capture {
    #[run]
    fn init() {}

    #[run_after(component(Rendering))]
    fn retrieve_buffer(&mut self, target: &TextureTarget, device: &GpuDevice) {
        self.buffer = target.retrieve_buffer(&device.device);
    }

    #[must_use]
    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }
}
