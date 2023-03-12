use crate::instances::opaque::OpaqueInstanceRegistry;
use crate::instances::transparent::TransparentInstanceRegistry;
use crate::render_target::RenderTargetRegistry;
use crate::resources::camera::Camera2DRegistry;
use crate::resources::material::MaterialRegistry;
use crate::resources::mesh::{Mesh, MeshRegistry};
use crate::resources::shader::{Shader, ShaderRegistry};
use crate::resources::texture::TextureRegistry;
use crate::Texture;
use futures::executor;
use instant::Instant;
use modor::{BuiltEntity, EntityBuilder};
use modor_input::InputModule;
use modor_physics::PhysicsModule;
use std::time::Duration;
use wgpu::{
    Adapter, Backends, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, BufferBindingType, Device, DeviceDescriptor, Instance, Limits, PresentMode, Queue,
    RequestAdapterOptions, SamplerBindingType, ShaderStages, TextureFormat, TextureSampleType,
    TextureViewDimension,
};

#[derive(SingletonComponent, Debug)]
pub struct GraphicsModule {
    pub frame_rate: FrameRate,
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) camera_bind_group_layout: BindGroupLayout,
    pub(crate) material_bind_group_layout: BindGroupLayout,
    pub(crate) texture_bind_group_layout: BindGroupLayout,
    pub(crate) window_texture_format: Option<TextureFormat>,
}

#[systems]
impl GraphicsModule {
    pub fn build() -> impl BuiltEntity {
        Self::build_with(FrameRate::VSync)
    }

    pub fn build_with(frame_rate: FrameRate) -> impl BuiltEntity {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(Backends::all);
        let instance = Instance::new(backends);
        let adapter_request = RequestAdapterOptions::default();
        let adapter = executor::block_on(instance.request_adapter(&adapter_request))
            .expect("no supported graphic adapter found");
        let (device, queue) = Self::retrieve_device(&adapter);
        let transparent_instances = TransparentInstanceRegistry::new(&device);
        EntityBuilder::new()
            .with(Self {
                frame_rate,
                instance,
                adapter,
                camera_bind_group_layout: Self::camera_bind_group_layout(&device),
                material_bind_group_layout: Self::material_bind_group_layout(&device),
                texture_bind_group_layout: Self::texture_bind_group_layout(&device),
                device,
                queue,
                window_texture_format: None,
            })
            .with_dependency::<PhysicsModule, _>(PhysicsModule::build())
            .with_dependency::<InputModule, _>(InputModule::build())
            .with_child(RenderTargetRegistry::new())
            .with_child(Camera2DRegistry::new())
            .with_child(ShaderRegistry::new())
            .with_child(MeshRegistry::new())
            .with_child(MaterialRegistry::new())
            .with_child(TextureRegistry::new())
            .with_child(OpaqueInstanceRegistry::default())
            .with_child(transparent_instances)
            .with_child(Shader::default())
            .with_child(Shader::ellipse())
            .with_child(Mesh::rectangle())
            .with_child(Texture::blank())
    }

    pub(crate) fn present_mode(&self, has_immediate_mode: bool) -> PresentMode {
        let is_vsync = matches!(self.frame_rate, FrameRate::VSync);
        if is_vsync || !has_immediate_mode {
            PresentMode::Fifo
        } else {
            PresentMode::Immediate
        }
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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FrameRate {
    Fps(u16),
    VSync,
    Unlimited,
}

impl FrameRate {
    pub(crate) fn run(&self, start: Instant, f: impl FnOnce()) {
        f();
        if let Self::Fps(frames_per_second) = self {
            if *frames_per_second > 0 {
                let update_time = Duration::from_secs_f32(1. / f32::from(*frames_per_second));
                let current_update_time = Instant::now().duration_since(start);
                if let Some(remaining_time) = update_time.checked_sub(current_update_time) {
                    if !cfg!(target_arch = "wasm32") {
                        spin_sleep::sleep(remaining_time);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod utils_tests {
    use crate::FrameRate;
    use instant::{Duration, Instant};

    #[test]
    fn run_with_frame_rate() {
        modor_internal::retry!(10, assert_duration(FrameRate::Unlimited, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::VSync, 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(0), 100, 100, 150));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(1), 500, 1000, 1200));
        modor_internal::retry!(10, assert_duration(FrameRate::Fps(5), 100, 200, 300));
    }

    fn assert_duration(
        frame_rate: FrameRate,
        external_sleep_millis: u64,
        min_millis: u64,
        max_millis: u64,
    ) {
        let update_start = Instant::now();
        frame_rate.run(Instant::now(), || {
            spin_sleep::sleep(Duration::from_millis(external_sleep_millis));
        });
        let update_end = Instant::now();
        assert!(update_end.duration_since(update_start) >= Duration::from_millis(min_millis));
        assert!(update_end.duration_since(update_start) <= Duration::from_millis(max_millis));
    }
}
