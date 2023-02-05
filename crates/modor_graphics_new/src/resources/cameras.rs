use crate::keys::cameras::CameraKey;
use crate::{CameraRef, Resolution};
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder, Query, Single};
use modor_input::{Finger, InputModule, Mouse};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::{PhysicsModule, Transform2D};

// TODO: make Camera2D a component + add with_* methods
// TODO: try to put camera key as generic type, corresponding to a component of the entity

// TODO: test camera deletion and duplicated keys

pub struct Camera2D {
    // excluded bound
    pub min_z: f32,
    // included bound
    pub max_z: f32,
    mouse_position: Vec2,
    finger_positions: FxHashMap<u64, Vec2>,
}

#[entity]
impl Camera2D {
    pub fn build(ref_: impl CameraRef) -> impl Built<Self> {
        EntityBuilder::new(Self {
            min_z: -0.5,
            max_z: 0.5,
            mouse_position: Vec2::new(0., 0.),
            finger_positions: FxHashMap::default(),
        })
        .inherit_from(Camera2DTransform::build(CameraKey::new(ref_)))
    }

    #[run_after(component(InputModule), component(Camera2DTransform))]
    fn update_mouse(
        &mut self,
        transform: &Camera2DTransform,
        mouse: Single<'_, Mouse>,
        resolution: Single<'_, Resolution>,
    ) {
        self.mouse_position = transform.window_to_world_position(mouse.position(), &resolution);
    }

    #[run_after(component(InputModule), component(Camera2DTransform))]
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
}

pub(crate) struct Camera2DTransform {
    key: CameraKey,
    window_to_world_matrix: Mat4,
    world_to_gpu_matrix: Mat4,
}

#[entity]
impl Camera2DTransform {
    fn build(key: CameraKey) -> impl Built<Self> {
        EntityBuilder::new(Self {
            key,
            window_to_world_matrix: Mat4::IDENTITY,
            world_to_gpu_matrix: Mat4::IDENTITY,
        })
        .with(Transform2D::new())
    }

    #[run_after(component(PhysicsModule))]
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

    pub(crate) fn key(&self) -> &CameraKey {
        &self.key
    }

    pub(crate) fn display_matrix(&self) -> Mat4 {
        self.world_to_gpu_matrix
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
        let z_diff = camera.max_z - camera.min_z;
        let position = Vec3::new(
            -transform.position.x,
            -transform.position.y,
            -camera.max_z / z_diff * z_diff,
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
