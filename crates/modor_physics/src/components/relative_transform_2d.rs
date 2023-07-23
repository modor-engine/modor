use modor_math::Vec2;

/// The relative positioning of an entity.
///
/// This component has an effect only if the entity and its parent have a component of type
/// [`Transform2D`](crate::Transform2D).
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
    /// Relative position of the entity in parent distance unit.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units
    /// (same along Y-axis).<br>
    /// The relative origin corresponds to the parent center.
    ///
    /// If [`None`], the absolute position of the [`Transform2D`](crate::Transform2D) component is
    /// taken into account.
    ///
    /// Default value is [`None`].
    pub position: Option<Vec2>,
    /// Relative size of the entity in parent distance unit.
    ///
    /// The parent distance unit is different than the world unit. A distance of `1.0` along the
    /// X-axis corresponds to the size along X-axis of the parent in world units (same along Y-axis).
    ///
    /// If [`None`], the absolute size of the [`Transform2D`](crate::Transform2D) component is taken
    /// into account.
    ///
    /// Default value is [`None`].
    pub size: Option<Vec2>,
    /// Relative rotation of the entity in radians.
    ///
    /// If [`None`], the absolute rotation of the [`Transform2D`](crate::Transform2D) component is
    /// taken into account.
    ///
    /// Default value is [`None`].
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
}
