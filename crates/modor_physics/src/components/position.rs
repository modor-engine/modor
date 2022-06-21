use crate::{RelativeVelocity, Rotation, Size, Velocity};
use modor_math::Vec3;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// The absolute position of an entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**:
///     - [`Velocity`](crate::Velocity), [`DeltaTime`](crate::DeltaTime)
///     - [`RelativePosition`](crate::RelativePosition), [`Size`](crate::Size),
///         [`Position`](crate::Position) of parent entity, [`Size`](crate::Size) of parent entity
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_math::{Vec3, Quat};
/// # use modor_physics::{
/// #     Acceleration, PhysicsModule, Position, Size, Shape, Velocity, Rotation, AngularVelocity,
/// #     AngularAcceleration
/// # };
/// #
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::from(Vec3::xy(0.2, 0.3)))
///             .with(Velocity::from(Vec3::xy(-0.01, 0.02)))
///             .with(Acceleration::from(Vec3::xy(0.5, -0.1)))
///             .with(Size::from(Vec3::xy(0.25, 0.5)))
///             .with(Rotation::from(Quat::from_z(20_f32.to_radians())))
///             .with(AngularVelocity::from(Quat::from_z(5_f32.to_radians())))
///             .with(AngularAcceleration::from(Quat::from_z(1_f32.to_radians())))
///             .with(Shape::Rectangle2D)
///     }
/// }
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct Position(Vec3);

impl Position {
    pub(crate) fn update_with_velocity(&mut self, velocity: Velocity, delta_time: Duration) {
        **self += *velocity * delta_time.as_secs_f32();
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_position: RelativePosition,
        parent_position: Self,
        parent_size: Size,
        parent_rotation: Rotation,
    ) {
        **self = relative_position.with_scale(*parent_size);
        **self = parent_rotation.matrix() * **self;
        **self += *parent_position;
    }
}

impl From<Vec3> for Position {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<Position> for Vec3 {
    fn from(position: Position) -> Self {
        position.0
    }
}

impl Deref for Position {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The relative position of an entity.
///
/// The position is relative to the first parent entity with a position and a size.<br>
/// In case the entity does not have any parent with a position and a size, the relative position is
/// equal to the absolute position.
///
/// A distance of `1.0` along an axis corresponds to the size along this axis of the first
/// parent having a position and a size.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](crate::Position), [`Size`](crate::Size)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeVelocity`](crate::RelativeVelocity),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_math::{Vec3, Quat};
/// # use modor_physics::{
/// #     Acceleration, PhysicsModule, Position, Size, Shape, Velocity, RelativeAcceleration,
/// #     RelativeVelocity, RelativePosition, RelativeSize, Rotation, RelativeRotation,
/// #     RelativeAngularVelocity, RelativeAngularAcceleration
/// # };
/// #
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::from(Vec3::ZERO))
///             .with(Size::from(Vec3::ZERO))
///             .with(Rotation::from(Quat::ZERO))
///             .with(RelativePosition::from(Vec3::xy(0.2, 0.3)))
///             .with(RelativeVelocity::from(Vec3::xy(-0.01, 0.02)))
///             .with(RelativeAcceleration::from(Vec3::xy(0.5, -0.1)))
///             .with(RelativeSize::from(Vec3::xy(0.25, 0.5)))
///             .with(RelativeRotation::from(Quat::from_z(20_f32.to_radians())))
///             .with(RelativeAngularVelocity::from(Quat::from_z(5_f32.to_radians())))
///             .with(RelativeAngularAcceleration::from(Quat::from_z(1_f32.to_radians())))
///             .with(Shape::Rectangle2D)
///     }
/// }
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativePosition(Vec3);

impl RelativePosition {
    pub(crate) fn update(&mut self, velocity: RelativeVelocity, delta_time: Duration) {
        **self += *velocity * delta_time.as_secs_f32();
    }
}

impl From<Vec3> for RelativePosition {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<RelativePosition> for Vec3 {
    fn from(position: RelativePosition) -> Self {
        position.0
    }
}

impl Deref for RelativePosition {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativePosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
