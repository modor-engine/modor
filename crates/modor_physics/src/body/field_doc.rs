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
    (damping) => {
        "Linear damping of the body.<br>\
        This coefficient is used to automatically slow down the translation of the body.<br>\
        Default is `0.0`."
    };
    (angular_damping) => {
        "Angular damping of the body.<br>\
        This coefficient is used to automatically slow down the rotation of the body.<br>\
        Default is `0.0`."
    };
    (dominance) => {
        "Dominance of the body.<br>\
        In case of collision between two bodies, if both bodies have a different dominance
        group, then collision forces will only be applied on the body with the smallest dominance.<br>\
        Has no effect if the [`collision_group`](Body2D::collision_group) is `None`.<br>\
        Default is `0`."
    };
    (is_ccd_enabled) => {
        "Whether Continuous Collision Detection is enabled for the body.<br>\
        This option is used to detect a collision even if the body moves too fast.
        CCD is performed using motion-clamping, which means each fast-moving body with CCD enabled
        will be stopped at the moment of their first contact. Both angular and translational motions
        are taken into account.<br>\
        Note that CCD require additional computation, so it is recommended to enable it only for
        bodies that are expected to move fast.<br>\
        Has no effect if [`collision_group`](#structfield.collision_group) is `None`.<br>\
        Default is `false`."
    };
    (shape) => {
        "The shape of the body used to detect collisions.<br>\
        Default is [`Shape2D::Rectangle`]."
    };
}

pub(super) use field_doc;
