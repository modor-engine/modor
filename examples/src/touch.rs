use modor::{
    systems, App, BuiltEntity, Component, Query, Single, SingleRef, SingletonComponent, World,
};
use modor_graphics::{
    instance_2d, window_target, Camera2D, Default2DMaterial, Window, WINDOW_CAMERA_2D,
};
use modor_input::Fingers;
use modor_math::Vec2;
use modor_physics::Transform2D;

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(FingerCreator)
        .run(modor_graphics::runner);
}

fn finger_display(finger_id: u64) -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, Default2DMaterial::new())
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.3)
        .updated(|m: &mut Default2DMaterial| m.is_ellipse = true)
        .component(FingerPosition::new(finger_id))
}

#[derive(SingletonComponent)]
struct FingerCreator;

#[systems]
impl FingerCreator {
    #[run_after(component(Fingers))]
    fn create_fingers(
        fingers: SingleRef<'_, '_, Fingers>,
        finger_positions: Query<'_, &FingerPosition>,
        mut world: World<'_>,
    ) {
        for finger_id in fingers.get().iter() {
            if !finger_positions.iter().any(|f| f.0 == finger_id) {
                world.create_root_entity(finger_display(finger_id));
            }
        }
    }
}

#[derive(Component)]
struct FingerPosition(u64, Option<Vec2>);

#[systems]
impl FingerPosition {
    fn new(finger_id: u64) -> Self {
        Self(finger_id, None)
    }

    #[run_after(component(Fingers))]
    fn retrieve(
        &mut self,
        fingers: SingleRef<'_, '_, Fingers>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) {
        let finger = &fingers.get()[self.0];
        self.1 = if finger.state.is_pressed() {
            let (window, camera) = window_camera.get();
            Some(camera.world_position(window.size(), finger.position))
        } else {
            None
        };
    }

    #[run_after_previous]
    fn update_display(&self, transform: &mut Transform2D, material: &mut Default2DMaterial) {
        if let Some(position) = self.1 {
            material.color.a = 1.;
            transform.position = position;
        } else {
            material.color.a = 0.;
        }
    }
}
