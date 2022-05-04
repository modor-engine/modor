/// The shape of an entity.
///
/// This component can be used by other modules, e.g. to know how to display the entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Required components**: [`Position`](crate::Position)
/// - **Default if missing**: `Shape::Rectangle2D`
///
/// # Examples
///
/// See [`PhysicsModule`](crate::PhysicsModule).
pub enum Shape {
    /// A 2D rectangle.
    ///
    /// Z-axis corresponds to the depth of the shape.
    Rectangle2D,
    /// A 2D circle.
    ///
    /// Z-axis corresponds to the depth of the shape.<br>
    /// If the width and the height of the entity are different, the shape is an ellipse.
    Circle2D,
}
