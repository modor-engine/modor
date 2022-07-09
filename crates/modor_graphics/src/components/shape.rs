/// The shape of an entity.
///
/// This component can be used by other modules, e.g. to know how to display the entity.
///
/// # Modor
///
/// - **Type**: component
/// - **Default if missing**: `Shape::Rectangle`
///
/// # Examples
///
/// See [`PhysicsModule`](modor_physics::PhysicsModule).
pub enum Shape {
    /// A rectangle.
    ///
    /// Z-axis corresponds to the depth of the shape.
    Rectangle,
    /// A circle.
    ///
    /// Z-axis corresponds to the depth of the shape.<br>
    /// If the width and the height of the entity are different, the shape is an ellipse.
    Circle,
}
