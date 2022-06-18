use crate::{RelativeVelocity, RelativeWorldUnit, Rotation, Size, Velocity, WorldUnit};
use modor_math::Point3D;
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
/// # use modor_physics::{Acceleration, PhysicsModule, Position, Size, Shape, Velocity};
/// #
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::xy(0.2, 0.3))
///             .with(Velocity::xy(-0.01, 0.02))
///             .with(Acceleration::xy(0.5, -0.1))
///             .with(Size::xy(0.25, 0.5))
///             .with(Shape::Rectangle2D)
///     }
/// }
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct Position(Point3D<WorldUnit>);

impl Position {
    /// An position with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a new position.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Point3D::xyz(x, y, z))
    }

    /// Creates a new position from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update_with_velocity(&mut self, velocity: Velocity, delta_time: Duration) {
        **self += *velocity * crate::Duration::from(delta_time);
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_position: RelativePosition,
        parent_position: Self,
        parent_size: Size,
        parent_rotation: Rotation,
    ) {
        self.x = relative_position.x * parent_size.x;
        self.y = relative_position.y * parent_size.y;
        self.z = relative_position.z * parent_size.z;
        **self = parent_rotation.matrix() * **self;
        self.x += parent_position.x;
        self.y += parent_position.y;
        self.z += parent_position.z;
    }
}

impl Deref for Position {
    type Target = Point3D<WorldUnit>;

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
/// - **Required components**: [`Position`](crate::Position)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeVelocity`](crate::RelativeVelocity),
///     [`DeltaTime`](crate::DeltaTime)
///
/// # Examples
///
/// ```rust
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_physics::{
/// #     Acceleration, PhysicsModule, Position, Size, Shape, Velocity, RelativeAcceleration,
/// #     RelativeVelocity, RelativePosition, RelativeSize
/// # };
/// #
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(Position::ZERO)
///             .with(Size::ONE)
///             .with(RelativePosition::xy(0.2, 0.3))
///             .with(RelativeVelocity::xy(-0.01, 0.02))
///             .with(RelativeAcceleration::xy(0.5, -0.1))
///             .with(RelativeSize::xy(0.25, 0.5))
///             .with(Shape::Rectangle2D)
///     }
/// }
/// ```
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativePosition(Point3D<RelativeWorldUnit>);

impl RelativePosition {
    /// An position with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a new position.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Point3D::xyz(x, y, z))
    }

    /// Creates a new position from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `0.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update(&mut self, velocity: RelativeVelocity, delta_time: Duration) {
        **self += *velocity * crate::Duration::from(delta_time);
    }
}

impl Deref for RelativePosition {
    type Target = Point3D<RelativeWorldUnit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativePosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
