use modor_math::Vec3;
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
pub struct Size(Vec3);

impl Size {
    pub(crate) fn update_with_relative(&mut self, relative_size: RelativeSize, parent_size: Self) {
        **self = relative_size.with_scale(*parent_size);
    }
}

impl From<Vec3> for Size {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<Size> for Vec3 {
    fn from(size: Size) -> Self {
        size.0
    }
}

impl Deref for Size {
    type Target = Vec3;

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
pub struct RelativeSize(Vec3);

impl From<Vec3> for RelativeSize {
    fn from(vector: Vec3) -> Self {
        Self(vector)
    }
}

impl From<RelativeSize> for Vec3 {
    fn from(size: RelativeSize) -> Self {
        size.0
    }
}

impl Deref for RelativeSize {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RelativeSize {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
