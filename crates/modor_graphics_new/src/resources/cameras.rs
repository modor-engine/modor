use crate::resources::shaders::Shader;
use crate::resources::uniforms::Uniform;
use crate::settings::rendering::Resolution;
use crate::targets::GpuDevice;
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder, Query, Single};
use modor_input::{Finger, InputModule, Mouse};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::{PhysicsModule, Transform2D};
use wgpu::RenderPass;

// TODO: allow user to create multiple Camera2D (default Camera2D is a non existing instance)
// TODO: handle camera removal
// TODO: handle camera duplication (same key for multiple cameras)
// TODO: handle Camera2D component switch (so key does not correspond to same entity ID)

pub struct Camera2D {
    pub min_z: f32,
    pub max_z: f32,
    mouse_position: Vec2,
    finger_positions: FxHashMap<u64, Vec2>,
    uniform: Uniform<CameraData>,
}

#[singleton]
impl Camera2D {
    pub(crate) fn build(uniform: Uniform<CameraData>) -> impl Built<Self> {
        EntityBuilder::new(Self {
            min_z: 0.,
            max_z: 1.,
            mouse_position: Vec2::new(0., 0.),
            finger_positions: FxHashMap::default(),
            uniform,
        })
        .inherit_from(Camera2DTransform::build())
    }

    #[run_after(entity(InputModule), entity(Camera2DTransform))]
    fn update_mouse(
        &mut self,
        transform: &Camera2DTransform,
        mouse: Single<'_, Mouse>,
        resolution: Single<'_, Resolution>,
    ) {
        self.mouse_position = transform.window_to_world_position(mouse.position(), &resolution);
    }

    #[run_after(entity(InputModule), entity(Camera2DTransform))]
    fn update_fingers(
        &mut self,
        transform: &Camera2DTransform,
        fingers: Query<'_, &Finger>,
        resolution: Single<'_, Resolution>,
    ) {
        self.finger_positions.clear();
        for finger in fingers.iter() {
            let position = transform.window_to_world_position(finger.position(), &resolution);
            self.finger_positions.insert(finger.id(), position);
        }
    }

    #[run_after(entity(Camera2DTransform))]
    fn update_uniform(&mut self, transform: &Camera2DTransform, device: Single<'_, GpuDevice>) {
        *self.uniform = CameraData {
            transform: transform.world_to_gpu_matrix.to_array(),
        };
        self.uniform.sync(&device);
    }

    #[must_use]
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    #[must_use]
    pub fn finger_position(&self, id: u64) -> Option<Vec2> {
        self.finger_positions.get(&id).copied()
    }

    pub fn finger_positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.finger_positions.values().copied()
    }

    pub(crate) fn use_for_rendering<'a>(&'a self, pass: &mut RenderPass<'a>) {
        self.uniform.use_for_rendering(Shader::CAMERA_GROUP, pass);
    }
}

pub struct Camera2DTransform {
    window_to_world_matrix: Mat4,
    world_to_gpu_matrix: Mat4,
}

#[singleton]
impl Camera2DTransform {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self {
            window_to_world_matrix: Mat4::IDENTITY,
            world_to_gpu_matrix: Mat4::IDENTITY,
        })
        .with(Transform2D::new())
    }

    #[run_after(entity(PhysicsModule))]
    fn update(
        &mut self,
        camera: &Camera2D,
        transform: &Transform2D,
        resolution: Single<'_, Resolution>,
    ) {
        let (width, height) = resolution.size_f32();
        let x_scale = 1.0_f32.min(height / width);
        let y_scale = 1.0_f32.min(width / height);
        self.window_to_world_matrix = Self::window_to_world_matrix(transform, x_scale, y_scale);
        self.world_to_gpu_matrix = Self::world_to_gpu_matrix(camera, transform, x_scale, y_scale);
    }

    fn window_to_world_position(&self, position: Vec2, resolution: &Resolution) -> Vec2 {
        let (width, height) = resolution.size_f32();
        self.window_to_world_matrix * Vec2::new(position.x / width - 0.5, 0.5 - position.y / height)
    }

    fn window_to_world_matrix(transform: &Transform2D, x_scale: f32, y_scale: f32) -> Mat4 {
        let position = Vec3::from_xy(transform.position.x, transform.position.y);
        let scale = Vec3::new(transform.size.x / x_scale, transform.size.y / y_scale, 1.);
        Mat4::from_scale(scale)
            * Quat::from_z(-*transform.rotation).matrix()
            * Mat4::from_position(position)
    }

    fn world_to_gpu_matrix(
        camera: &Camera2D,
        transform: &Transform2D,
        x_scale: f32,
        y_scale: f32,
    ) -> Mat4 {
        let z_scale = 1. / (camera.max_z - camera.min_z);
        let position = Vec3::new(-transform.position.x, -transform.position.y, -camera.min_z);
        let scale = Vec3::new(
            2. * x_scale / transform.size.x,
            2. * y_scale / transform.size.y,
            z_scale,
        );
        Mat4::from_position(position)
            * Quat::from_z(*transform.rotation).matrix()
            * Mat4::from_scale(scale)
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub(crate) struct CameraData {
    transform: [[f32; 4]; 4],
}
