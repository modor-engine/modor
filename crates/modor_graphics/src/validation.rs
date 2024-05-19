use crate::gpu::Gpu;
use futures::executor;
use wgpu::ErrorFilter;

pub(crate) fn validate_wgpu<T>(gpu: &Gpu, f: impl FnOnce() -> T) -> Result<T, wgpu::Error> {
    gpu.device.push_error_scope(ErrorFilter::Validation);
    let value = f();
    executor::block_on(gpu.device.pop_error_scope()).map_or(Ok(value), |e| Err(e))
}
