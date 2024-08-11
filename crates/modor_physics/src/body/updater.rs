use crate::body::convert_vector2;
use crate::user_data::ColliderUserData;
use crate::{body, Body2D, Body2DUpdater, Shape2D};
use modor::{App, Glob, Update};
use rapier2d::dynamics::{MassProperties, RigidBody};
use rapier2d::geometry::{Collider, SharedShape};
use rapier2d::math::Rotation;
use rapier2d::na::Point2;

impl Body2DUpdater<'_> {
    /// Runs the update.
    pub fn apply(mut self, app: &mut App, glob: &Glob<Body2D>) {
        glob.take(app, |body, app| {
            let (rigid_body, collider) = body
                .pipeline
                .get_mut(app)
                .rigid_body_and_collider_mut(body.rigid_body_handle, body.collider_handle);
            self.update_collision_group(glob, body, collider);
            self.update_position(rigid_body);
            self.update_size_and_shape(body, collider);
            self.update_rotation(rigid_body);
            self.update_velocity(rigid_body);
            self.update_angular_velocity(rigid_body);
            self.update_force(rigid_body);
            self.update_torque(rigid_body);
            self.update_mass_and_angular_inertia(body, rigid_body);
            self.update_damping(body, rigid_body);
            self.update_angular_damping(body, rigid_body);
            self.update_dominance(body, rigid_body);
            self.update_ccd_enabled(body, rigid_body);
        });
    }

    fn update_collision_group(
        &mut self,
        glob: &Glob<Body2D>,
        body: &mut Body2D,
        collider: &mut Collider,
    ) {
        if Update::apply_checked(&mut self.collision_group, &mut body.collision_group) {
            let group_index = body
                .collision_group
                .as_ref()
                .map_or(usize::MAX, |group| group.index());
            collider.user_data = ColliderUserData::new(glob.index(), group_index).into();
            collider.set_enabled(body.collision_group.is_some());
        }
    }

    fn update_position(&mut self, rigid_body: &mut RigidBody) {
        if let Some(position) = self
            .position
            .take_value(|| convert_vector2(*rigid_body.translation()))
        {
            rigid_body.set_translation(body::convert_vec2(position), true);
        }
    }

    fn update_size_and_shape(&mut self, body: &mut Body2D, collider: &mut Collider) {
        if Update::apply_checked(&mut self.size, &mut body.size)
            | Update::apply_checked(&mut self.shape, &mut body.shape)
        {
            collider.set_shape(match body.shape {
                Shape2D::Rectangle => SharedShape::cuboid(body.size.x / 2., body.size.y / 2.),
                Shape2D::Circle => SharedShape::ball(body.size.x.min(body.size.y) / 2.),
            });
            collider.set_mass(0.);
        }
    }

    fn update_rotation(&mut self, rigid_body: &mut RigidBody) {
        if let Some(rotation) = self.rotation.take_value(|| rigid_body.rotation().angle()) {
            rigid_body.set_rotation(Rotation::new(rotation), true);
        }
    }

    fn update_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(velocity) = self
            .velocity
            .take_value(|| convert_vector2(*rigid_body.linvel()))
        {
            rigid_body.set_linvel(body::convert_vec2(velocity), true);
        }
    }

    fn update_angular_velocity(&mut self, rigid_body: &mut RigidBody) {
        if let Some(angular_velocity) = self.angular_velocity.take_value(|| rigid_body.angvel()) {
            rigid_body.set_angvel(angular_velocity, true);
        }
    }

    fn update_force(&mut self, rigid_body: &mut RigidBody) {
        if let Some(force) = self
            .force
            .take_value(|| convert_vector2(rigid_body.user_force()))
        {
            rigid_body.reset_forces(true);
            rigid_body.add_force(body::convert_vec2(force), true);
        }
    }

    fn update_torque(&mut self, rigid_body: &mut RigidBody) {
        if let Some(torque) = self.torque.take_value(|| rigid_body.user_torque()) {
            rigid_body.reset_torques(true);
            rigid_body.add_torque(torque, true);
        }
    }

    fn update_mass_and_angular_inertia(&mut self, body: &mut Body2D, rigid_body: &mut RigidBody) {
        if Update::apply_checked(&mut self.mass, &mut body.mass)
            | Update::apply_checked(&mut self.angular_inertia, &mut body.angular_inertia)
        {
            let properties =
                MassProperties::new(Point2::new(0., 0.), body.mass, body.angular_inertia);
            rigid_body.set_additional_mass_properties(properties, true);
        }
    }

    fn update_damping(&mut self, body: &mut Body2D, rigid_body: &mut RigidBody) {
        if Update::apply_checked(&mut self.damping, &mut body.damping) {
            rigid_body.set_linear_damping(body.damping);
        }
    }

    fn update_angular_damping(&mut self, body: &mut Body2D, rigid_body: &mut RigidBody) {
        if Update::apply_checked(&mut self.angular_damping, &mut body.angular_damping) {
            rigid_body.set_angular_damping(body.angular_damping);
        }
    }

    fn update_dominance(&mut self, body: &mut Body2D, rigid_body: &mut RigidBody) {
        if Update::apply_checked(&mut self.dominance, &mut body.dominance) {
            rigid_body.set_dominance_group(body.dominance);
        }
    }

    fn update_ccd_enabled(&mut self, body: &mut Body2D, rigid_body: &mut RigidBody) {
        if Update::apply_checked(&mut self.is_ccd_enabled, &mut body.is_ccd_enabled) {
            rigid_body.enable_ccd(body.is_ccd_enabled);
        }
    }

    // fn update_body(&self, body: &mut Body2D) {
    //     let collision_group = self.collision_group.clone();
    //     modor::update_field(&mut body.collision_group, collision_group);
    //     modor::update_field(&mut body.size, self.size);
    //     modor::update_field(&mut body.mass, self.mass);
    //     modor::update_field(&mut body.angular_inertia, self.angular_inertia);
    // }
    //
    // fn update_pipeline(&mut self, app: &mut App, body: &Body2D) {
    //     let (rigid_body, collider) = body
    //         .pipeline
    //         .get_mut(app)
    //         .rigid_body_and_collider_mut(body.rigid_body_handle, body.collider_handle);
    //     self.update_collision_group(collider);
    //     self.update_position(rigid_body);
    //     self.update_size(collider);
    //     self.update_rotation(rigid_body);
    //     self.update_velocity(rigid_body);
    //     self.update_angular_velocity(rigid_body);
    //     self.update_force(rigid_body);
    //     self.update_torque(rigid_body);
    //     self.update_mass_and_angular_inertia(rigid_body, body);
    //     self.update_damping(rigid_body);
    //     self.update_angular_damping(rigid_body);
    //     self.update_dominance(rigid_body);
    //     self.update_ccd_enabled(rigid_body);
    //     self.update_shape(collider, body);
    // }
}
