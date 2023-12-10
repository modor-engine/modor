use crate::components::renderer::GpuContext;
use futures::executor;
use wgpu::ErrorFilter;

pub(crate) fn validate_wgpu<T>(
    context: &GpuContext,
    f: impl FnOnce() -> T,
) -> Result<T, wgpu::Error> {
    context.device.push_error_scope(ErrorFilter::Validation);
    let value = f();
    executor::block_on(context.device.pop_error_scope()).map_or(Ok(value), |e| Err(e))
}
