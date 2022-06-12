use crate::{RelativeVelocity, Rotation, Size, Velocity};
use modor_math::{Point3D, Quaternion};
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
#[derive(Clone, Copy, Debug)]
pub struct Position {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Position {
    /// A position with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D position.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D position.
    ///
    /// Z-coordinate is set to `0.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update_with_velocity(&mut self, velocity: Velocity, delta_time: Duration) {
        self.x += velocity.x * delta_time.as_secs_f32();
        self.y += velocity.y * delta_time.as_secs_f32();
        self.z += velocity.z * delta_time.as_secs_f32();
    }

    pub(crate) fn update_with_relative(
        &mut self,
        relative_position: RelativePosition,
        parent_position: Self,
        parent_size: Size,
        parent_rotation: Rotation,
    ) {
        self.x = relative_position
            .x
            .mul_add(parent_size.x, parent_position.x)
            - parent_position.x;
        self.y = relative_position
            .y
            .mul_add(parent_size.y, parent_position.y)
            - parent_position.y;
        self.z = relative_position
            .z
            .mul_add(parent_size.z, parent_position.z)
            - parent_position.z;
        *self = modor_math::multiply_matrix_and_point_3d(*self, parent_rotation.matrix());
        self.x += parent_position.x;
        self.y += parent_position.y;
        self.z += parent_position.z;
    }
}

impl Point3D for Position {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

/// The relative position of an entity.
///
/// The position is relative to the first parent entity with a position and a size.<br>
/// In case the entity does not have any parent with a position and a size, the relative position is
/// equal to the absolute position.
///
/// A distance of 1 along an axis corresponds to the size along this axis of the first
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
#[derive(Clone, Copy, Debug)]
pub struct RelativePosition {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl RelativePosition {
    /// A position with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D position.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D position.
    ///
    /// Z-coordinate is set to `0.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }

    pub(crate) fn update(&mut self, velocity: RelativeVelocity, delta_time: Duration) {
        self.x += velocity.x * delta_time.as_secs_f32();
        self.y += velocity.y * delta_time.as_secs_f32();
        self.z += velocity.z * delta_time.as_secs_f32();
    }
}

impl Point3D for RelativePosition {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
