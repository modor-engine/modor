use modor::{Built, EntityBuilder};
use modor_physics::{Position, Scale};
use std::marker::PhantomData;

/// The camera used for 2D rendering.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
/// - **Default if missing**: `Camera2D::build(Position::xy(0., 0.), Scale::xy(1., 1.))`
/// - **Inner components**: [`Position`](modor_physics::Position),
///     [`Scale`](modor_physics::Scale)
///
/// # Examples
/// ```rust
/// # use modor::App;
/// # use modor_physics::{Position, Scale};
/// # use modor_graphics::Camera2D;
/// #
/// App::new()
///     .with_entity(Camera2D::build(Position::xy(0.5, 0.7), Scale::xy(2., 2.)));
/// ```
pub struct Camera2D(PhantomData<()>);

#[singleton]
impl Camera2D {
    /// Builds the entity.
    pub fn build(position: Position, scale: Scale) -> impl Built<Self> {
        EntityBuilder::new(Self(PhantomData))
            .with(position)
            .with(scale)
    }
}
