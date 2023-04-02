use futures::executor;
use modor::Single;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use wgpu::{
    Adapter, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, Device, DeviceDescriptor, Instance, Limits, PowerPreference,
    Queue, RequestAdapterOptions, SamplerBindingType, ShaderStages, Surface, TextureFormat,
    TextureSampleType, TextureViewDimension,
};

static RENDERER_VERSION: AtomicU8 = AtomicU8::new(0);

#[derive(SingletonComponent, Debug)]
pub struct Renderer {
    pub(crate) version: Option<u8>,
    context: Option<Arc<GpuContext>>,
}

#[systems]
impl Renderer {
    pub(crate) fn new() -> Self {
        Self {
            version: None,
            context: None,
        }
    }

    #[run]
    fn init(&mut self) {
        if self.version.is_none() {
            self.version = Some(RENDERER_VERSION.fetch_add(1, Ordering::AcqRel));
        }
        if self.context.is_none() {
            let instance = GpuContext::instance();
            self.context = Some(Arc::new(GpuContext::new(&instance, None)));
        }
    }

    pub(crate) fn update(&mut self, renderer: &Arc<GpuContext>) {
        if self.context.is_none() {
            self.context = Some(renderer.clone());
        }
    }

    pub(crate) fn state(&self, last_version: &mut Option<u8>) -> RendererState<'_> {
        if let Some(context) = &self.context {
            let version = self
                .version
                .expect("internal error: version not assigned to renderer");
            *last_version = Some(version);
            if last_version == &Some(version) {
                RendererState::Unchanged(context)
            } else {
                RendererState::Changed(context)
            }
        } else {
            *last_version = None;
            RendererState::None
        }
    }

    pub(crate) fn option_state<'a>(
        renderer: &'a Option<Single<'_, Self>>,
        renderer_version: &mut Option<u8>,
    ) -> RendererState<'a> {
        renderer
            .as_ref()
            .map_or(RendererState::None, |r| r.state(renderer_version))
    }
}

#[derive(Debug)]
pub struct GpuContext {
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) camera_bind_group_layout: BindGroupLayout,
    pub(crate) material_bind_group_layout: BindGroupLayout,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    pub(crate) surface_texture_format: Option<TextureFormat>,
}

impl GpuContext {
    pub(crate) fn new(instance: &Instance, surface: Option<&Surface>) -> Self {
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
            material_bind_group_layout: Self::material_bind_group_layout(&device),
            texture_bind_group_layout: Self::texture_bind_group_layout(&device),
            surface_texture_format: surface
                .and_then(|s| s.get_supported_formats(&adapter).into_iter().next()),
            adapter,
            device,
            queue,
        }
    }

    pub(crate) fn instance() -> Instance {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all);
        Instance::new(backends)
    }

    fn retrieve_device(adapter: &Adapter) -> (Device, Queue) {
        let device_descriptor = DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: Self::gpu_limits(),
            label: None,
        };
        executor::block_on(adapter.request_device(&device_descriptor, None))
            .expect("error when retrieving graphic device")
    }

    fn gpu_limits() -> Limits {
        if cfg!(target_arch = "wasm32") {
            Limits::downlevel_webgl2_defaults()
        } else {
            Limits::default()
        }
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

    fn material_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX_FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("modor_bind_group_layout_material"),
        })
    }

    fn texture_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("modor_bind_group_layout_texture"),
        })
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) enum RendererState<'a> {
    #[default]
    None,
    Unchanged(&'a GpuContext),
    Changed(&'a GpuContext),
}

impl<'a> RendererState<'a> {
    pub(crate) fn context(self) -> Option<&'a GpuContext> {
        if let Self::Changed(renderer) | Self::Unchanged(renderer) = self {
            Some(renderer)
        } else {
            None
        }
    }

    pub(crate) fn is_removed(self) -> bool {
        matches!(self, Self::None | Self::Changed(_))
    }
}
