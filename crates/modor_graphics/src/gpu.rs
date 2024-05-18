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
    pub(crate) current_version: u64,
    pub(crate) instance: Arc<Instance>,
    details: Option<Arc<Gpu>>,
}

impl RootNode for GpuManager {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self {
            current_version: 1,
            instance: Self::create_instance().into(),
            details: None,
        }
    }
}

impl GpuManager {
    // TODO: is Arc<> still necessary ?
    pub(crate) fn get(&mut self) -> &Arc<Gpu> {
        self.details
            .get_or_insert_with(|| Gpu::new(&self.instance, None, self.current_version).into())
    }

    pub(crate) fn configure_window(&mut self, surface: &Surface<'_>) {
        self.details = Some(Gpu::new(&self.instance, Some(surface), self.current_version).into());
    }

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
    pub(crate) version: u64,
}

impl Gpu {
    fn new(instance: &Instance, surface: Option<&Surface<'_>>, version: u64) -> Self {
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
            version,
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

#[derive(Debug, Default)]
pub(crate) struct GpuHandle {
    version: u64,
}

impl GpuHandle {
    pub(crate) fn get(&mut self, ctx: &mut Context<'_>) -> GpuState {
        let manager = ctx.root::<GpuManager>().get_mut(ctx);
        let gpu = manager
            .details
            .get_or_insert_with(|| {
                Gpu::new(&manager.instance, None, manager.current_version).into()
            })
            .clone();
        if gpu.version == self.version {
            GpuState::Same(gpu)
        } else {
            self.version = gpu.version;
            GpuState::New(gpu)
        }
    }

    pub(crate) fn action(&mut self, ctx: &mut Context<'_>, is_loaded: bool) -> GpuResourceAction {
        match (self.get(ctx), is_loaded) {
            (GpuState::None, _) => GpuResourceAction::Delete,
            (GpuState::New(gpu) | GpuState::Same(gpu), true) | (GpuState::New(gpu), false) => {
                GpuResourceAction::Create(gpu)
            }
            (GpuState::Same(gpu), false) => GpuResourceAction::Update(gpu),
        }
    }
}

pub(crate) enum GpuState {
    None,
    New(Arc<Gpu>),
    Same(Arc<Gpu>),
}

impl GpuState {
    pub(crate) fn take(self) -> Option<Arc<Gpu>> {
        match self {
            Self::None => None,
            Self::New(gpu) | Self::Same(gpu) => Some(gpu),
        }
    }
}

pub(crate) enum GpuResourceAction {
    Delete,
    Create(Arc<Gpu>),
    Update(Arc<Gpu>),
}
