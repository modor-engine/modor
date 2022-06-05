use modor_math::Vector3D;

/// The absolute acceleration of an entity.
///
/// The acceleration is measured in distance units per second squared.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `Acceleration::xyz(0., 0., 0.)`
/// - **Required components**: [`Velocity`](crate::Velocity)
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct Acceleration {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Acceleration {
    /// An acceleration with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D acceleration.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D acceleration.
    ///
    /// Z-coordinate is set to zero.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }
}

impl Vector3D for Acceleration {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self::xyz(x, y, z)
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}

/// The relative acceleration of an entity.
///
/// The acceleration is measured in distance units per second squared.<br>
/// A distance unit along 1 on an axis corresponds to the size along this axis of the first
/// parent having a position and a size.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `RelativeAcceleration::xyz(0., 0., 0.)`
/// - **Required components**: [`RelativeVelocity`](crate::RelativeVelocity)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Clone, Copy, Debug, Add, Sub, AddAssign, SubAssign)]
pub struct RelativeAcceleration {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl RelativeAcceleration {
    /// An acceleration with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);

    /// Creates a 3D acceleration.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D acceleration.
    ///
    /// Z-coordinate is set to zero.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 0.)
    }
}

impl Vector3D for RelativeAcceleration {
    fn create(x: f32, y: f32, z: f32) -> Self {
        Self::xyz(x, y, z)
    }

    fn components(self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }
}
