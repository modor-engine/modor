/// The Z-index of a rendered 2D instance.
///
/// It is created from a [`u16`] value, where `0` is the farthest from the camera,
/// and [`u16::MAX`] the closest to the camera.
///
/// By default, the z-index of an instance is `0`.
///
/// # Requirements
///
/// The component is effective only if:
/// - [`Transform2D`](modor_physics::Transform2D) component is in the same entity
/// - An [`InstanceGroup2D`](crate::InstanceGroup2D) is linked to the entity
///
/// # Related components
///
/// - [`Transform2D`](modor_physics::Transform2D)
/// - [`InstanceGroup2D`](crate::InstanceGroup2D)
///
/// # Examples
///
/// ```rust
/// # use modor::*;
/// # use modor_graphics::*;
/// # use modor_math::*;
/// # use modor_physics::*;
/// # use modor_resources::*;
/// #
/// const CAMERA: ResKey<Camera2D> = ResKey::new("main");
/// const FOREGROUND_MATERIAL: ResKey<Material> = ResKey::new("foreground");
/// const BACKGROUND_MATERIAL: ResKey<Material> = ResKey::new("background");
///
/// fn foreground() -> impl BuiltEntity {
///     instance_2d::<Default2DMaterial>(CAMERA, None)
///         .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.5)
///         .component(ZIndex2D::from(1))
/// }
///
/// fn background() -> impl BuiltEntity {
///     instance_2d::<Default2DMaterial>(CAMERA, None)
///         .updated(|t: &mut Transform2D| t.size = Vec2::ONE)
///         .component(ZIndex2D::from(0))
/// }
/// ```
#[must_use]
#[derive(
    Component, NoSystem, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
pub struct ZIndex2D(u16);

impl ZIndex2D {
    // Returns the instance depth between `0.` and `1.`.
    pub(crate) fn to_normalized_f32(self) -> f32 {
        (f32::from(self.0) + 0.5) / (f32::from(u16::MAX) + 1.)
    }
}

impl From<u16> for ZIndex2D {
    fn from(index: u16) -> Self {
        Self(index)
    }
}

impl From<ZIndex2D> for u16 {
    fn from(index: ZIndex2D) -> Self {
        index.0
    }
}
