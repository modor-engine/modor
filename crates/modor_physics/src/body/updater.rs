use crate::user_data::ColliderUserData;
use crate::{body, Body2D, Body2DUpdater, Shape2D};
use modor::App;
use rapier2d::dynamics::{MassProperties, RigidBody};
use rapier2d::geometry::{Collider, SharedShape};
use rapier2d::math::Rotation;
use rapier2d::na::Point2;

impl Body2DUpdater<'_> {
    /// Run the update.
    pub fn apply(mut self, app: &mut App) {
        self.glob.take(app, |body, app| {
            self.update_body(body);
            self.update_pipeline(app, body);
        });
    }

    fn update_body(&self, body: &mut Body2D) {
        let collision_group = self.collision_group.clone();
        modor::update_field(&mut body.collision_group, collision_group, &mut false);
        modor::update_field(&mut body.size, self.size, &mut false);
        modor::update_field(&mut body.mass, self.mass, &mut false);
        modor::update_field(&mut body.angular_inertia, self.angular_inertia, &mut false);
    }

    fn update_pipeline(&mut self, app: &mut App, body: &Body2D) {
        let (rigid_body, collider) = body
            .pipeline
            .get_mut(app)
            .rigid_body_and_collider_mut(body.rigid_body_handle, body.collider_handle);
        self.update_collision_group(collider);
        self.update_position(rigid_body);
        self.update_size(collider);
        self.update_rotation(rigid_body);
        self.update_velocity(rigid_body);
        self.update_angular_velocity(rigid_body);
        self.update_force(rigid_body);
        self.update_torque(rigid_body);
        self.update_mass_and_angular_inertia(rigid_body, body);
        self.update_damping(rigid_body);
        self.update_angular_damping(rigid_body);
        self.update_dominance(rigid_body);
        self.update_ccd_enabled(rigid_body);
        self.update_shape(collider, body);
    }

    fn update_collision_group(&mut self, collider: &mut Collider) {
        if let Some(collision_group) = self.collision_group.take() {
            let group_index = collision_group
                .as_ref()
                .map_or(usize::MAX, |group| group.index());
            collider.user_data = ColliderUserData::new(self.glob.index(), group_index).into();
            collider.set_enabled(collision_group.is_some());
        }
    }

    fn update_position(&mut self, rigid_body: &mut RigidBody) {
        if let Some(position) = self.position {
            rigid_body.set_translation(body::convert_vec2(position), true);
        }
    }

    fn update_size(&mut self, collider: &mut Collider) {
        if let Some(size) = self.size {
            let shape = collider.shape_mut();
            if let Some(shape) = shape.as_cuboid_mut() {
                shape.half_extents = body::convert_vec2(size / 2.);
            } else if let Some(shape) = shape.as_ball_mut() {
                shape.radius = size.x.min(size.y) / 2.;
            } else {
                unreachable!("internal error: unsupported body shape")
            }
            collider.set_mass(0.);
        }
    }

    fn update_rotation(&mut self, rigid_body: &mut RigidBody) {
        if let Some(rotation) = self.rotation {
            rigid_body.set_rotation(Rotation::new(rotation), true);
        }
    }

    fn update_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(velocity) = self.velocity {
            rigid_body.set_linvel(body::convert_vec2(velocity), true);
        }
    }

    fn update_angular_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(angular_velocity) = self.angular_velocity {
            rigid_body.set_angvel(angular_velocity, true);
        }
    }

    fn update_force(&mut self, rigid_body: &mut RigidBody) {
        if let Some(force) = self.force {
            rigid_body.reset_forces(true);
            rigid_body.add_force(body::convert_vec2(force), true);
        }
    }

    fn update_torque(&mut self, rigid_body: &mut RigidBody) {
        if let Some(torque) = self.torque {
            rigid_body.reset_torques(true);
            rigid_body.add_torque(torque, true);
        }
    }

    fn update_mass_and_angular_inertia(&mut self, rigid_body: &mut RigidBody, body: &Body2D) {
        if self.mass.is_some() || self.angular_inertia.is_some() {
            let properties =
                MassProperties::new(Point2::new(0., 0.), body.mass, body.angular_inertia);
            rigid_body.set_additional_mass_properties(properties, true);
        }
    }

    fn update_damping(&mut self, rigid_body: &mut RigidBody) {
        if let Some(damping) = self.damping {
            rigid_body.set_linear_damping(damping);
        }
    }

    fn update_angular_damping(&mut self, rigid_body: &mut RigidBody) {
        if let Some(angular_damping) = self.angular_damping {
            rigid_body.set_angular_damping(angular_damping);
        }
    }

    fn update_dominance(&mut self, rigid_body: &mut RigidBody) {
        if let Some(dominance) = self.dominance {
            rigid_body.set_dominance_group(dominance);
        }
    }

    fn update_ccd_enabled(&mut self, rigid_body: &mut RigidBody) {
        if let Some(is_ccd_enabled) = self.is_ccd_enabled {
            rigid_body.enable_ccd(is_ccd_enabled);
        }
    }

    fn update_shape(&mut self, collider: &mut Collider, body: &Body2D) {
        if let Some(shape) = self.shape {
            collider.set_shape(match shape {
                Shape2D::Rectangle => SharedShape::cuboid(body.size.x / 2., body.size.y / 2.),
                Shape2D::Circle => SharedShape::ball(body.size.x.min(body.size.y) / 2.),
            });
            collider.set_mass(0.);
        }
    }
}
