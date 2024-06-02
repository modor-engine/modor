use crate::platform;
use futures::executor;
use modor::{Context, Node, RootNode, Visit};
use std::sync::Arc;
use wgpu::{
    Adapter, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, Device, DeviceDescriptor, Features, Gles3MinorVersion,
    Instance, InstanceFlags, PowerPreference, Queue, RequestAdapterOptions, ShaderStages, Surface,
};

#[derive(Debug, Visit, Node)]
pub(crate) struct GpuManager {
    pub(crate) instance: Arc<Instance>,
    details: Option<Arc<Gpu>>,
}

impl RootNode for GpuManager {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self {
            instance: Self::create_instance().into(),
            details: None,
        }
    }
}

impl GpuManager {
    pub(crate) fn get_or_init(&mut self) -> &Arc<Gpu> {
        self.details
            .get_or_insert_with(|| Gpu::new(&self.instance, None).into())
    }

    pub(crate) fn get(&self) -> Option<&Arc<Gpu>> {
        self.details.as_ref()
    }

    // coverage: off (window cannot be tested)
    pub(crate) fn configure_window(&mut self, surface: &Surface<'_>) {
        self.details = Some(Gpu::new(&self.instance, Some(surface)).into());
    }
    // coverage: on

    fn create_instance() -> Instance {
        Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all),
            flags: InstanceFlags::default(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            gles_minor_version: Gles3MinorVersion::Automatic,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Gpu {
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) camera_bind_group_layout: BindGroupLayout,
}

impl Gpu {
    fn new(instance: &Instance, surface: Option<&Surface<'_>>) -> Self {
        let adapter_request = RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: surface,
        };
        let adapter = executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found");
        let (device, queue) = Self::retrieve_device(&adapter);
        Self {
            camera_bind_group_layout: Self::camera_bind_group_layout(&device),
            adapter,
            device,
            queue,
        }
    }

    fn retrieve_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            label: None,
            required_features: Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
            required_limits: platform::gpu_limits(),
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }

    fn camera_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("modor_bind_group_layout_camera"),
        })
    }
}
