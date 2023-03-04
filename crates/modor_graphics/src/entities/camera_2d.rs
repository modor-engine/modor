use crate::utils::numbers;
use crate::Window;
use fxhash::FxHashMap;
use modor::{Built, EntityBuilder, Query, Single};
use modor_input::{Finger, InputModule, Mouse};
use modor_math::{Mat4, Quat, Vec2, Vec3};
use modor_physics::{PhysicsModule, Transform2D};

/// The camera used for 2D rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: same as [`GraphicsModule`](crate::GraphicsModule)
/// - **Default if missing**: `Camera2D::build(Position::xy(0., 0.), Size::xy(1., 1.))`
/// - **Inner components**: [`Transform2D`](modor_physics::Transform2D)
/// - **Updated using**: [`Transform2D`](modor_physics::Transform2D), [`Mouse`](modor_input::Mouse),
///     [`Finger`](modor_input::Finger), [`Window`](crate::Window)
///
/// # Examples
/// ```rust
/// # use modor::{App, Single};
/// # use modor_math::Vec2;
/// # use modor_graphics::Camera2D;
/// #
/// App::new()
///     .with_entity(Camera2D::build(Vec2::new(0.5, 0.7), Vec2::new(2., 2.)));
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
    pub fn build(position: Vec2, size: Vec2) -> impl Built<Self> {
        EntityBuilder::new(Self {
            transform_matrix: Mat4::IDENTITY,
            mouse_position: Vec2::new(0., 0.),
            finger_positions: FxHashMap::default(),
        })
        .with(Transform2D::new().with_position(position).with_size(size))
    }

    /// Builds the entity with a rotation.
    pub fn build_rotated(position: Vec2, size: Vec2, rotation: f32) -> impl Built<Self> {
        EntityBuilder::new(Self {
            transform_matrix: Mat4::IDENTITY,
            mouse_position: Vec2::new(0., 0.),
            finger_positions: FxHashMap::default(),
        })
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(size)
                .with_rotation(rotation),
        )
    }

    // coverage: off (window cannot be tested)

    /// Returns the 2D world position of the mouse.
    ///
    /// Does not work in windowless mode.
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Returns the 2D world position of the finger with ID `ìd`.
    ///
    /// Does not work in windowless mode.
    pub fn finger_position(&self, id: u64) -> Option<Vec2> {
        self.finger_positions.get(&id).copied()
    }

    /// Returns an iterator on all finger positions.
    pub fn finger_positions(&self) -> impl Iterator<Item = Vec2> + '_ {
        self.finger_positions.values().copied()
    }

    #[run_after(component(PhysicsModule), component(InputModule))]
    fn update_matrix(&mut self, transform: &Transform2D, window: Single<'_, Window>) {
        let (x_scale, y_scale) = numbers::world_scale((window.size().width, window.size().height));
        let position = Vec3::from_xy(transform.position.x, transform.position.y);
        let scale = Vec3::new(transform.size.x / x_scale, transform.size.y / y_scale, 1.);
        let rotation = -1. * *transform.rotation;
        self.transform_matrix = Mat4::from_scale(scale)
            * Quat::from_z(rotation).matrix()
            * Mat4::from_position(position);
    }

    #[run_after_previous]
    fn update_from_mouse(&mut self, mouse: Single<'_, Mouse>, window: Single<'_, Window>) {
        self.mouse_position =
            self.transform_matrix * Self::window_to_backend_coordinates(mouse.position(), &window);
    }

    #[run_after_previous]
    fn update_from_fingers(&mut self, fingers: Query<'_, &Finger>, window: Single<'_, Window>) {
        self.finger_positions.clear();
        self.finger_positions.extend(fingers.iter().map(|f| {
            (
                f.id(),
                self.transform_matrix * Self::window_to_backend_coordinates(f.position(), &window),
            )
        }));
    }

    #[allow(clippy::cast_precision_loss)]
    fn window_to_backend_coordinates(position: Vec2, window: &Window) -> Vec2 {
        Vec2::new(
            position.x / window.size().width as f32 - 0.5,
            0.5 - position.y / window.size().height as f32,
        )
    }

    // coverage: on
}
