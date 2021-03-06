use crate::entities::camera::internal::UpdateCamera2DMatrixAction;
use crate::{utils, Window};
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder, Query, Single};
use modor_input::{Finger, Mouse, UpdateInputAction};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::{Transform, UpdatePhysicsAction};

/// The camera used for 2D rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `Camera2D::build(Position::xy(0., 0.), Size::xy(1., 1.))`
/// - **Inner components**: [`Transform`](modor_physics::Transform)
/// - **Updated during**: [`UpdateCamera2DAction`](crate::UpdateCamera2DAction)
/// - **Updated using**: [`Transform`](modor_physics::Transform), [`Mouse`](modor_input::Mouse),
///     [`Finger`](modor_input::Finger), [`Window`](crate::Window)
///
/// # Examples
/// ```rust
/// # use modor::{App, Single};
/// # use modor_math::Vec3;
/// # use modor_graphics::Camera2D;
/// #
/// App::new()
///     .with_entity(Camera2D::build(Vec3::xy(0.5, 0.7), Vec3::xy(2., 2.)));
///
/// fn access_mouse_position(camera: Single<'_, Camera2D>) {
///     println!("Mouse position in 2D world: {:?}", camera.mouse_position());
/// }
/// ```
pub struct Camera2D {
    transform_matrix: Mat4,
    mouse_position: Vec2,
    finger_positions: FxHashMap<u64, Vec2>,
}

#[singleton]
impl Camera2D {
    /// Builds the entity.
    pub fn build(position: Vec3, size: Vec3) -> impl Built<Self> {
        EntityBuilder::new(Self {
            transform_matrix: Mat4::IDENTITY,
            mouse_position: Vec2::xy(0., 0.),
            finger_positions: FxHashMap::default(),
        })
        .with(Transform::new().with_position(position).with_size(size))
    }

    /// Builds the entity with a rotation.
    pub fn build_rotated(position: Vec3, size: Vec3, rotation: Quat) -> impl Built<Self> {
        EntityBuilder::new(Self {
            transform_matrix: Mat4::IDENTITY,
            mouse_position: Vec2::xy(0., 0.),
            finger_positions: FxHashMap::default(),
        })
        .with(
            Transform::new()
                .with_position(position)
                .with_size(size)
                .with_rotation(rotation),
        )
    }

    // coverage: off (window cannot be tested)

    /// Returns the 2D world position of the mouse.
    ///
    /// Does not work in windowless mode.
    #[must_use]
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Returns the 2D world position of the finger with ID `??d`.
    ///
    /// Does not work in windowless mode.
    #[must_use]
    pub fn finger_position(&self, id: u64) -> Option<Vec2> {
        self.finger_positions.get(&id).copied()
    }

    /// Returns an iterator on all finger positions.
    pub fn finger_positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.finger_positions.values().copied()
    }

    #[run_as(UpdateCamera2DMatrixAction)]
    fn update_matrix(&mut self, transform: &Transform, window: Single<'_, Window>) {
        let (x_scale, y_scale) = utils::world_scale((window.size().width, window.size().height));
        let position = Vec3::xy(transform.position.x, transform.position.y);
        let scale = Vec3::xyz(transform.size.x / x_scale, transform.size.y / y_scale, 1.);
        let rotation = transform.rotation.with_scale(-1.);
        self.transform_matrix =
            Mat4::from_scale(scale) * rotation.matrix() * Mat4::from_position(position);
    }

    #[run_as(UpdateCamera2DAction)]
    fn update_from_mouse(&mut self, mouse: Single<'_, Mouse>, window: Single<'_, Window>) {
        self.mouse_position =
            self.transform_matrix * Self::window_to_backend_coordinates(mouse.position(), &*window);
    }

    #[run_as(UpdateCamera2DAction)]
    fn update_from_fingers(&mut self, fingers: Query<'_, &Finger>, window: Single<'_, Window>) {
        self.finger_positions.clear();
        self.finger_positions.extend(fingers.iter().map(|f| {
            (
                f.id(),
                self.transform_matrix * Self::window_to_backend_coordinates(f.position(), &*window),
            )
        }));
    }

    #[allow(clippy::cast_precision_loss)]
    fn window_to_backend_coordinates(position: Vec2, window: &Window) -> Vec2 {
        Vec2::xy(
            position.x / window.size().width as f32 - 0.5,
            0.5 - position.y / window.size().height as f32,
        )
    }

    // coverage: on
}

/// An action done when the [`Camera2D`](crate::Camera2D) has been updated.
#[action(UpdateCamera2DMatrixAction, UpdatePhysicsAction, UpdateInputAction)]
pub struct UpdateCamera2DAction;

mod internal {
    #[action]
    pub struct UpdateCamera2DMatrixAction;
}
