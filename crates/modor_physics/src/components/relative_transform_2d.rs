use modor_math::Vec2;

/// The relative positioning of an entity.
///
/// This component has an effect only if the entity has also a component of type
/// [`Transform2D`](crate::Transform2D).
///
/// The parent taken into account is the first parent in the entity hierarchy that has a
/// [`Transform2D`](crate::Transform2D) component.
///
/// Only the properties different than `None` are relative to the parent.
/// The equivalent properties of the [`Transform2D`](crate::Transform2D) are automatically updated.
///
/// [`Dynamics2D`](crate::Dynamics2D) will have no effect with this component.
///
/// # Example
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[non_exhaustive]
#[derive(Clone, Debug, Default, Component, NoSystem)]
pub struct RelativeTransform2D {
    /// Relative position of the entity.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units
    /// (same along Y-axis).<br>
    /// The relative origin corresponds to the parent center.
    ///
    /// If `None`, the absolute position of the [`Transform2D`](crate::Transform2D) component is
    /// taken into account.
    pub position: Option<Vec2>,
    /// Relative size of the entity in parent distance unit.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units (same along Y-axis).
    ///
    /// If `None`, the absolute size of the [`Transform2D`](crate::Transform2D) component is taken
    /// into account.
    pub size: Option<Vec2>,
    /// Relative rotation of the entity in radians.
    ///
    /// If `None`, the absolute rotation of the [`Transform2D`](crate::Transform2D) component is
    /// taken into account.
    pub rotation: Option<f32>,
}

impl RelativeTransform2D {
    /// Creates a new transform.
    #[inline]
    pub const fn new() -> Self {
        Self {
            position: None,
            size: None,
            rotation: None,
        }
    }

    /// Returns the transform with a different `position` in parent distance units.
    ///
    /// By default, the position is not relative.
    #[inline]
    pub const fn with_position(mut self, position: Vec2) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the transform with a different `size` in parent distance units.
    ///
    /// By default, the size is not relative.
    #[inline]
    pub const fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    /// Returns the transform with a different `rotation` in radians.
    ///
    /// By default, the rotation is not relative.
    #[inline]
    pub const fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }
}
