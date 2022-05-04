/// The scale of an entity.
///
/// The scale is relative to the first parent entity having a position and a scale.<br>
/// This is an absolute size in case the entity does not have any parent with a position and a
/// scale.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](crate::Position)
/// - **Updated by**: [`PhysicsModule`](crate::PhysicsModule)
/// - **Updated using**: [`Scale`](crate::Scale) of parent entity
/// - **Updated during**: [`UpdatePhysicsAction`](crate::UpdatePhysicsAction)
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
#[derive(Clone, Debug)]
pub struct Scale {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
    abs: Size,
}

impl Scale {
    /// Creates a 3D scale.
    ///
    /// Absolute size is initialized with the same coordinates.
    pub const fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            abs: Size { x, y, z },
        }
    }

    /// Creates a 2D scale.
    ///
    /// Z-coordinate is set to zero.
    ///
    /// Absolute size is initialized with the same coordinates.
    pub const fn xy(x: f32, y: f32) -> Self {
        Self::xyz(x, y, 1.)
    }

    /// Returns the absolute size.
    pub fn abs(&self) -> &Size {
        &self.abs
    }

    pub(crate) fn update_abs(&mut self, parent_scale: &Self) {
        self.abs.x = parent_scale.abs.x * self.x;
        self.abs.y = parent_scale.abs.y * self.y;
        self.abs.z = parent_scale.abs.z * self.z;
    }
}

/// An absolute size corresponding to a relative [`Scale`](crate::Scale).
#[derive(Clone, Debug)]
pub struct Size {
    /// The X-coordinate.
    pub x: f32,
    /// The Y-coordinate.
    pub y: f32,
    /// The Z-coordinate.
    pub z: f32,
}

#[cfg(test)]
mod scale_tests {
    use crate::Scale;
    use approx::assert_abs_diff_eq;

    #[test]
    fn create_2d() {
        let scale = Scale::xy(4., 2.);
        assert_abs_diff_eq!(scale.x, 4.);
        assert_abs_diff_eq!(scale.y, 2.);
        assert_abs_diff_eq!(scale.z, 1.);
        assert_abs_diff_eq!(scale.abs().x, 4.);
        assert_abs_diff_eq!(scale.abs().y, 2.);
        assert_abs_diff_eq!(scale.abs().z, 1.);
    }

    #[test]
    fn create_3d() {
        let scale = Scale::xyz(1., 2., 3.);
        assert_abs_diff_eq!(scale.x, 1.);
        assert_abs_diff_eq!(scale.y, 2.);
        assert_abs_diff_eq!(scale.z, 3.);
        assert_abs_diff_eq!(scale.abs().x, 1.);
        assert_abs_diff_eq!(scale.abs().y, 2.);
        assert_abs_diff_eq!(scale.abs().z, 3.);
    }
}
