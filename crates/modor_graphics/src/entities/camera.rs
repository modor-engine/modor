use crate::{utils, Window};
use modor::{Built, EntityBuilder, Single};
use modor_input::{Mouse, UpdateInputAction};
use modor_physics::{Position, Size, UpdatePhysicsAction};

/// The camera used for 2D rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `Camera2D::build(Position::xy(0., 0.), Size::xy(1., 1.))`
/// - **Inner components**: [`Position`](modor_physics::Position), [`Size`](modor_physics::Size)
/// - **Updated during**: [`UpdateCamera2DAction`](crate::UpdateCamera2DAction)
/// - **Updated using**: [`Position`](modor_physics::Position), [`Size`](modor_physics::Size),
///     [`Mouse`](modor_input::Mouse), [`Window`](crate::Window)
///
/// # Examples
/// ```rust
/// # use modor::{App, Single};
/// # use modor_physics::{Position, Size};
/// # use modor_graphics::Camera2D;
/// #
/// App::new()
///     .with_entity(Camera2D::build(Position::xy(0.5, 0.7), Size::xy(2., 2.)));
///
/// fn access_mouse_position(camera: Single<'_, Camera2D>) {
///     println!("Mouse position in 2D world: {:?}", camera.mouse_position());
/// }
/// ```
pub struct Camera2D {
    mouse_position: Position,
}

#[singleton]
impl Camera2D {
    /// Builds the entity.
    pub fn build(position: Position, size: Size) -> impl Built<Self> {
        EntityBuilder::new(Self {
            mouse_position: Position::xy(0., 0.),
        })
        .with(position)
        .with(size)
    }

    // coverage: off (window cannot be tested)
    /// Returns the 2D world position of the mouse.
    ///
    /// Does not work in windowless mode.
    pub fn mouse_position(&self) -> Position {
        self.mouse_position
    }
    // coverage: on

    // coverage: off (window cannot be tested)
    #[allow(clippy::cast_precision_loss)]
    #[run_as(UpdateCamera2DAction)]
    fn update_from_mouse(
        &mut self,
        position: &Position,
        size: &Size,
        mouse: Single<'_, Mouse>,
        window: Single<'_, Window>,
    ) {
        let (x_scale, y_scale) = utils::world_scale((window.size().width, window.size().height));
        self.mouse_position.x = ((mouse.position().x / window.size().width as f32 - 0.5) / x_scale)
            .mul_add(size.x, position.x);
        self.mouse_position.y = ((0.5 - mouse.position().y / window.size().height as f32)
            / y_scale)
            .mul_add(size.y, position.y);
    }
    // coverage: on
}

/// An action done when the [`Camera2D`](crate::Camera2D) has been updated.
#[action(UpdatePhysicsAction, UpdateInputAction)]
pub struct UpdateCamera2DAction;
