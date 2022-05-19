pub(crate) mod texture;
pub(crate) mod window;

use crate::utils;
use std::any::Any;
use wgpu::{
    Adapter, CommandEncoder, Device, DeviceDescriptor, Features, Limits, Queue, TextureFormat,
    TextureView,
};

pub(crate) trait Target: Any + Sync + Send {
    fn size(&self) -> (u32, u32);

    fn texture_format(&self) -> TextureFormat;

    fn retrieve_buffer(&self, device: &Device) -> Vec<u8>;

    fn set_size(&mut self, width: u32, height: u32, device: &Device);

    fn toggle_vsync(&mut self, enabled: bool, device: &Device);

    fn prepare_texture(&mut self) -> TextureView;

    fn render(&mut self, queue: &Queue, encoder: CommandEncoder);
}

pub(crate) struct CreatedTarget<T> {
    pub(super) target: T,
    pub(super) device: Device,
    pub(super) queue: Queue,
}

fn retrieve_device(adapter: &Adapter) -> (Device, Queue) {
    utils::block_on(adapter.request_device(
        &DeviceDescriptor {
            features: Features::empty(),
            limits: {
                #[cfg(target_arch = "wasm32")]
                {
                    Limits::downlevel_webgl2_defaults()
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    Limits::default()
                }
            },
            label: None,
        },
        None,
    ))
    .expect("error when retrieving device")
}
