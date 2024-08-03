use crate::collisions::Collision2D;
use crate::physics_hooks::PhysicsHooks;
use crate::user_data::ColliderUserData;
use crate::{Body2D, Delta};
use modor::{App, FromApp, Globals, State};
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhaseMultiSap, Collider, ColliderHandle, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::RigidBody;
use std::mem;

#[derive(FromApp)]
pub(crate) struct Pipeline {
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    island_manager: IslandManager,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    integration_parameters: IntegrationParameters,
    #[allow(clippy::struct_field_names)]
    physics_pipeline: PhysicsPipeline,
    broad_phase: BroadPhaseMultiSap,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    collisions: Vec<Vec<Collision2D>>,
}

impl State for Pipeline {
    fn update(&mut self, app: &mut App) {
        for (_, body) in app.get_mut::<Globals<Body2D>>().deleted_items() {
            self.rigid_bodies.remove(
                body.rigid_body_handle,
                &mut self.island_manager,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
        }
        self.update_collision_groups(app);
        self.integration_parameters.dt = app.get_mut::<Delta>().duration.as_secs_f32();
        self.physics_pipeline.step(
            &Vector2::zeros(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            app.get_mut::<PhysicsHooks>(),
            &(),
        );
        self.reset_collisions();
        self.register_collisions();
        self.send_collisions(app);
    }
}

impl Pipeline {
    pub(crate) fn rigid_body(&self, handle: RigidBodyHandle) -> &RigidBody {
        &self.rigid_bodies[handle]
    }

    pub(crate) fn collider(&self, handle: ColliderHandle) -> &Collider {
        &self.colliders[handle]
    }

    pub(crate) fn rigid_body_and_collider_mut(
        &mut self,
        body_handle: RigidBodyHandle,
        collider_handle: ColliderHandle,
    ) -> (&mut RigidBody, &mut Collider) {
        (
            &mut self.rigid_bodies[body_handle],
            &mut self.colliders[collider_handle],
        )
    }

    pub(crate) fn register_body(
        &mut self,
        rigid_body: RigidBody,
        collider: Collider,
    ) -> (RigidBodyHandle, ColliderHandle) {
        let rigid_body_handle = self.rigid_bodies.insert(rigid_body);
        let collider_handle =
            self.colliders
                .insert_with_parent(collider, rigid_body_handle, &mut self.rigid_bodies);
        (rigid_body_handle, collider_handle)
    }

    fn update_collision_groups(&mut self, app: &mut App) {
        app.take::<PhysicsHooks, _>(|hooks, app| {
            for body in app.get_mut::<Globals<Body2D>>().iter_mut() {
                if let Some(group) = body.collision_group() {
                    let group_index = group.index();
                    let groups = hooks.interaction_groups[group_index];
                    self.colliders[body.collider_handle].set_collision_groups(groups);
                }
            }
        });
    }

    fn reset_collisions(&mut self) {
        for collisions in &mut self.collisions {
            collisions.clear();
        }
    }

    fn register_collisions(&mut self) {
        for pair in self.narrow_phase.contact_pairs() {
            let collider1 = &self.colliders[pair.collider1];
            let collider2 = &self.colliders[pair.collider2];
            let body1 = ColliderUserData::from(collider1.user_data);
            let body2 = ColliderUserData::from(collider2.user_data);
            let body1_index = body1.body_index();
            let body2_index = body2.body_index();
            (self.collisions.len()..=body1_index.max(body2_index))
                .for_each(|_| self.collisions.push(vec![]));
            for manifold in &pair.manifolds {
                if manifold.points.iter().all(|p| p.dist > 0.) {
                    continue;
                }
                self.collisions[body1_index].push(Collision2D::new(
                    false,
                    body2_index,
                    body2.group_index(),
                    collider1,
                    manifold,
                ));
                self.collisions[body2_index].push(Collision2D::new(
                    true,
                    body1_index,
                    body1.group_index(),
                    collider2,
                    manifold,
                ));
            }
        }
    }

    fn send_collisions(&mut self, app: &mut App) {
        for (index, body) in app.get_mut::<Globals<Body2D>>().iter_mut_enumerated() {
            body.collisions = self
                .collisions
                .get_mut(index)
                .map(mem::take)
                .unwrap_or_default();
        }
    }
}
