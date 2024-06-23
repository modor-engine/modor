use crate::gpu::Gpu;
use futures::executor;
use log::LevelFilter;
use wgpu::ErrorFilter;

pub(crate) fn validate_wgpu<T>(gpu: &Gpu, f: impl FnOnce() -> T) -> Result<T, wgpu::Error> {
    gpu.device.push_error_scope(ErrorFilter::Validation);
    let log_level = log::max_level();
    log::set_max_level(LevelFilter::Off);
    let value = f();
    log::set_max_level(log_level);
    executor::block_on(gpu.device.pop_error_scope()).map_or(Ok(value), |e| Err(e))
}
