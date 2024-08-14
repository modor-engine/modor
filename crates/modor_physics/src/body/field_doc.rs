macro_rules! field_doc {
    (position) => {
        "Position of the body in world units.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (rotation) => {
        "Rotation of the body in radians.<br>\
        Default is `0.0`."
    };
    (velocity) => {
        "Linear velocity of the body in world units per second.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (angular_velocity) => {
        "Angular velocity of the body in radians per second.<br>\
        Has no effect if the [`angular_inertia`](Body2D::angular_inertia) is `0.0`.<br>\
        Default is `0.0`."
    };
    (force) => {
        "Force applied on the body.<br>\
        Has no effect if the [`mass`](Body2D::mass) is `0.0`.<br>\
        The acceleration of the body corresponds to the force of the body divided by its mass.<br>\
        Default is [`Vec2::ZERO`]."
    };
    (torque) => {
        "Torque applied on the body.<br>\
        Has no effect if the [`angular_inertia`](Body2D::angular_inertia) is `0.0`.<br>\
        Default is `0.0`."
    };
}

pub(super) use field_doc;
