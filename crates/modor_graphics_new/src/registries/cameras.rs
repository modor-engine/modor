use crate::keys::cameras::{CameraKey, DefaultCameraRef};
use crate::resources::cameras::Camera2D;
use crate::resources::cameras::Camera2DTransform;
use crate::resources::uniforms::Uniform;
use crate::targets::{GpuDevice, Target, CAMERA_BINDING};
use fxhash::FxHashMap;
use log::error;
use modor::{Built, EntityBuilder, Query, Single};

pub(crate) struct Camera2DRegistry {
    cameras: FxHashMap<CameraKey, CameraDetails>,
    default_key: CameraKey,
}

#[singleton]
impl Camera2DRegistry {
    pub(crate) fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            cameras: FxHashMap::default(),
            default_key: CameraKey::new(DefaultCameraRef),
        })
        .with_child(Camera2D::build(DefaultCameraRef))
    }

    #[run_after(component(Camera2DTransform))]
    fn update(
        &mut self,
        cameras: Query<'_, &Camera2DTransform>,
        target: Single<'_, Target>,
        device: Single<'_, GpuDevice>,
    ) {
        for camera in self.cameras.values_mut() {
            camera.is_updated = false;
        }
        for transform in cameras.iter() {
            self.update_uniform(transform, &target, &device);
        }
        self.cameras.retain(|_, c| c.is_updated);
        for camera in self.cameras.values_mut() {
            camera.uniform.sync(&device);
        }
    }

    pub(crate) fn uniform(&self, key: &CameraKey) -> &Uniform<CameraData> {
        self.cameras.get(key).map_or_else(
            || {
                error!("not created camera with reference '{:?}'", key);
                self.default_uniform()
            },
            |c| &c.uniform,
        )
    }

    fn default_uniform(&self) -> &Uniform<CameraData> {
        &self
            .cameras
            .get(&self.default_key)
            .expect("internal error: default camera not found")
            .uniform
    }

    fn update_uniform(
        &mut self,
        transform: &Camera2DTransform,
        target: &Target,
        device: &GpuDevice,
    ) {
        let key = transform.key();
        let data = CameraData {
            transform: transform.display_matrix().to_array(),
        };
        if let Some(camera) = self.cameras.get_mut(key) {
            if camera.is_updated {
                error!("multiple cameras have the same reference '{:?}'", key);
            }
            *camera.uniform = data;
            camera.is_updated = true;
        } else {
            let camera = CameraDetails::new(data, target, device);
            self.cameras.insert(key.clone(), camera);
        }
    }
}

struct CameraDetails {
    uniform: Uniform<CameraData>,
    is_updated: bool,
}

impl CameraDetails {
    fn new(data: CameraData, target: &Target, device: &GpuDevice) -> Self {
        Self {
            uniform: Uniform::new(
                data,
                CAMERA_BINDING,
                target.camera_bind_group_layout(),
                "camera",
                &device.device,
            ),
            is_updated: true,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct CameraData {
    transform: [[f32; 4]; 4],
}
