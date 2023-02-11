use crate::rendering::CameraData;
use crate::resources::registries::{Resource, ResourceRegistry};
use crate::resources::uniforms::Uniform;
use crate::targets::{GpuDevice, Target, CAMERA_BINDING};
use crate::{Resolution, ResourceKey};
use log::trace;
use modor::{Built, Changed, ChildBuilder, EntityBuilder, Filter, Or, Single};
use modor_internal::dyn_types::DynType;
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::Transform2D;
use std::fmt::Debug;
use std::hash::Hash;

pub(crate) type Camera2DRegistry = ResourceRegistry<Camera2D>;
type ChangedCamera2D = Or<(Changed<Camera2D>, Changed<Transform2D>, Changed<Resolution>)>;

pub struct Camera2D {
    // excluded bound
    pub min_z: f32,
    // included bound
    pub max_z: f32,
    key: DynType,
    world_matrix: Mat4,
    resolution: (f32, f32),
    uniform: Option<Uniform<CameraData>>,
}

#[component]
impl Camera2D {
    pub fn new(key: impl ResourceKey) -> Self {
        Self {
            key: DynType::new(key),
            min_z: -0.5,
            max_z: 0.5,
            world_matrix: Mat4::IDENTITY,
            resolution: (1., 1.),
            uniform: None,
        }
    }

    #[must_use]
    pub fn with_min_z(mut self, min_z: f32) -> Self {
        self.min_z = min_z;
        self
    }

    #[must_use]
    pub fn with_max_z(mut self, max_z: f32) -> Self {
        self.max_z = max_z;
        self
    }

    #[run_after(component(Transform2D))]
    fn update(
        &mut self,
        transform: &Transform2D,
        resolution: Single<'_, Resolution>,
        device: Single<'_, GpuDevice>,
        target: Single<'_, Target>,
        _: Filter<ChangedCamera2D>,
    ) {
        self.resolution = resolution.size_f32();
        self.world_matrix = self.world_matrix(transform);
        self.update_uniform(&device, &target, transform);
        if let Some(uniform) = &mut self.uniform {
            uniform.sync(&device);
        }
        trace!("Camera2D '{:?}' updated", self.key);
    }

    #[must_use]
    pub fn world_position(&self, window_position: Vec2) -> Vec2 {
        let (width, height) = self.resolution;
        self.world_matrix
            * Vec2::new(
                window_position.x / width - 0.5,
                0.5 - window_position.y / height,
            )
    }

    pub(crate) fn uniform(&self) -> &Uniform<CameraData> {
        self.uniform
            .as_ref()
            .expect("internal error: camera uniform not initialized")
    }

    fn update_uniform(&mut self, device: &GpuDevice, target: &Target, transform: &Transform2D) {
        let data = CameraData {
            transform: self.gpu_matrix(transform).to_array(),
        };
        if let Some(uniform) = &mut self.uniform {
            **uniform = data;
        } else {
            self.uniform = Some(Uniform::new(
                data,
                CAMERA_BINDING,
                target.camera_bind_group_layout(),
                "camera_2d",
                &device.device,
            ));
        }
    }

    fn world_matrix(&self, transform: &Transform2D) -> Mat4 {
        let (width, height) = self.resolution;
        let x_scale = 1.0_f32.min(height / width);
        let y_scale = 1.0_f32.min(width / height);
        let position = Vec3::from_xy(transform.position.x, transform.position.y);
        let scale = Vec3::new(transform.size.x / x_scale, transform.size.y / y_scale, 1.);
        Mat4::from_scale(scale)
            * Quat::from_z(-*transform.rotation).matrix()
            * Mat4::from_position(position)
    }

    fn gpu_matrix(&self, transform: &Transform2D) -> Mat4 {
        let (width, height) = self.resolution;
        let x_scale = 1.0_f32.min(height / width);
        let y_scale = 1.0_f32.min(width / height);
        let z_diff = self.max_z - self.min_z;
        let position = Vec3::new(
            -transform.position.x,
            -transform.position.y,
            -self.max_z / z_diff * z_diff,
        );
        let scale = Vec3::new(
            2. * x_scale / transform.size.x,
            2. * y_scale / transform.size.y,
            1. / -z_diff,
        );
        Mat4::from_position(position)
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_scale(scale)
    }
}

impl Resource for Camera2D {
    fn key(&self) -> &DynType {
        &self.key
    }

    fn build_default(builder: &mut ChildBuilder<'_>) {
        struct DefaultCamera2D;

        #[singleton]
        impl DefaultCamera2D {
            fn build() -> impl Built<Self> {
                EntityBuilder::new(Self)
                    .with(Transform2D::new())
                    .with(Camera2D::new(DefaultCameraKey))
            }
        }

        builder.add(DefaultCamera2D::build());
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DefaultCameraKey;
