use crate::{RelativeWorldUnit, WorldUnit};
use modor_math::Size3D;
use std::ops::{Deref, DerefMut};

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
#[derive(Default, Clone, Copy, Debug)]
pub struct Size(Size3D<WorldUnit>);

impl Size {
    /// A size with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);
    /// A size with all components equal to `1.0`.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// Creates a new size.
    #[inline]
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Size3D::xyz(x, y, z))
    }

    /// Creates a new size from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `1.0`.
    #[inline]
    pub const fn xy(x: f32, y: f32) -> Self {
        Self(Size3D::xyz(x, y, 1.)) // TODO: replace z by 0.0 + update doc
    }

    pub(crate) fn update_with_relative(&mut self, relative_size: RelativeSize, parent_size: Self) {
        self.x = relative_size.x * parent_size.x;
        self.y = relative_size.y * parent_size.y;
        self.z = relative_size.z * parent_size.z;
    }
}

impl Deref for Size {
    type Target = Size3D<WorldUnit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Size {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
#[derive(Default, Clone, Copy, Debug)]
pub struct RelativeSize(Size3D<RelativeWorldUnit>);

impl RelativeSize {
    /// A size with all components equal to `0.0`.
    pub const ZERO: Self = Self::xyz(0., 0., 0.);
    /// A size with all components equal to `1.0`.
    pub const ONE: Self = Self::xyz(1., 1., 1.);

    /// Creates a 3D size.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self(Size3D::xyz(x, y, z))
    }

    /// Creates a new size from 2D coordinates.
    ///
    /// Z-coordinate is initialized to `1.0`.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 1.) // TODO: replace z by 0.0 + update doc
    }
}

impl Deref for RelativeSize {
    type Target = Size3D<RelativeWorldUnit>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeSize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
