use crate::Window;
use modor::{Built, EntityBuilder, Single};
use modor_input::{Mouse, UpdateInputAction};
use modor_physics::{Position, Scale, UpdatePhysicsAction};

/// The camera used for 2D rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `Camera2D::build(Position::xy(0., 0.), Scale::xy(1., 1.))`
/// - **Updated during**: [`UpdateCamera2DAction`](crate::UpdateCamera2DAction)
/// - **Inner components**: [`Position`](modor_physics::Position),
///     [`Scale`](modor_physics::Scale)
///
/// # Examples
/// ```rust
/// # use modor::{App, Single};
/// # use modor_physics::{Position, Scale};
/// # use modor_graphics::Camera2D;
/// #
/// App::new()
///     .with_entity(Camera2D::build(Position::xy(0.5, 0.7), Scale::xy(2., 2.)));
///
/// fn access_mouse_position(camera: Single<'_, Camera2D>) {
///     println!("Mouse position in 2D world: {:?}", camera.mouse_position());
/// }
/// ```
pub struct Camera2D {
    mouse_position: Mouse2DWorldPosition,
}

#[singleton]
impl Camera2D {
    /// Builds the entity.
    pub fn build(position: Position, scale: Scale) -> impl Built<Self> {
        EntityBuilder::new(Self {
            mouse_position: Mouse2DWorldPosition { x: 0., y: 0. },
        })
        .with(position)
        .with(scale)
    }

    /// Returns the 2D world position of the mouse.
    pub fn mouse_position(&self) -> Mouse2DWorldPosition {
        self.mouse_position
    }

    #[allow(clippy::cast_precision_loss)]
    #[run_as(UpdateCamera2DAction)]
    fn update_from_mouse(
        &mut self,
        position: &Position,
        scale: &Scale,
        mouse: Single<'_, Mouse>,
        window: Single<'_, Window>,
    ) {
        // TODO: avoid code duplication
        let x_scale = f32::min(window.size().height as f32 / window.size().width as f32, 1.);
        let y_scale = f32::min(window.size().width as f32 / window.size().height as f32, 1.);
        self.mouse_position.x = (mouse.position().x / window.size().width as f32 - 0.5 / x_scale)
            .mul_add(scale.abs().x, position.abs().x);
        self.mouse_position.y = (0.5 - mouse.position().y / window.size().height as f32 / y_scale)
            .mul_add(scale.abs().y, position.abs().y);
    }
}

/// The 2D world position of the mouse.
///
/// # Examples
///
/// See [`Camera2D`](crate::Camera2D).
#[derive(Clone, Copy, Debug)]
pub struct Mouse2DWorldPosition {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
}

/// An action done when the [`Camera2D`](crate::Camera2D) has been updated.
#[action(UpdatePhysicsAction, UpdateInputAction)]
pub struct UpdateCamera2DAction;
