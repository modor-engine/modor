/// The absolute size of an entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](crate::Position)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`RelativeSize`](crate::RelativeSize), [`Size`](crate::Size) of parent
///     entity
///
/// # Examples
///
/// See [`Position`](crate::Position).
#[derive(Clone, Copy, Debug)]
pub struct Size {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl Size {
    /// A size with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);
    /// A size with all components equal to one.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// Creates a 3D size.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D size.
    ///
    /// Z-coordinate is set to `1.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 1.)
    }

    pub(crate) fn update_with_relative(&mut self, relative_size: RelativeSize, parent_size: Self) {
        self.x = relative_size.x * parent_size.x;
        self.y = relative_size.y * parent_size.y;
        self.z = relative_size.z * parent_size.z;
    }
}

/// The relative size of an entity.
///
/// The size is relative to the first parent entity having a size.<br>
/// This is an absolute size in case the entity does not have any parent with a size.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Size`](crate::Size)
///
/// # Examples
///
/// See [`RelativePosition`](crate::RelativePosition).
#[derive(Clone, Copy, Debug)]
pub struct RelativeSize {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

impl RelativeSize {
    /// A size with all components equal to zero.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);
    /// A size with all components equal to one.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// Creates a 3D size.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Creates a 2D size.
    ///
    /// Z-coordinate is set to `1.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 1.)
    }
}
