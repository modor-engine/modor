use modor_math::{Quat, Vec3};
use std::marker::PhantomData;

/// The relative positioning of an entity.
///
/// The parent taken into account is the first parent in the entity hierarchy than has a
/// `Transform` component.
///
/// Only the properties different than `None` are relative to the parent.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Transform`](crate::Transform)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
/// - **Updated using**: [`DynamicBody`](crate::DynamicBody), [`DeltaTime`](crate::DeltaTime)
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug, Default)]
pub struct RelativeTransform {
    /// Relative position of the entity.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units
    /// (same along Y-axis).<br>
    /// The relative origin corresponds to the parent center.
    ///
    /// If `None`, the absolute position of the `Transform` component is taken into account.
    pub position: Option<Vec3>,
    /// Relative size of the entity in parent distance unit.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units (same along Y-axis).
    ///
    /// If `None`, the absolute size of the `Transform` component is taken into account.
    pub size: Option<Vec3>,
    /// Relative rotation of the entity in radians.
    ///
    /// If `None`, the absolute rotation of the `Transform` component is taken into account.
    pub rotation: Option<Quat>,
    phantom: PhantomData<()>,
}

impl RelativeTransform {
    /// Creates a new transform.
    #[must_use]
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: None,
            size: None,
            rotation: None,
            phantom: PhantomData,
        }
    }

    /// Returns the transform with a different `position`.
    #[must_use]
    #[inline]
    pub const fn with_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the transform with a different `size`.
    #[must_use]
    #[inline]
    pub const fn with_size(mut self, size: Vec3) -> Self {
        self.size = Some(size);
        self
    }

    /// Returns the transform with a different `rotation`.
    #[must_use]
    #[inline]
    pub const fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = Some(rotation);
        self
    }
}
