use super::collision_groups::CollisionGroupStorage;
use crate::storages_2d::bodies::BodyStorage;
use crate::storages_2d::colliders::ColliderStorage;
use crate::utils::UserData;
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
};
use rapier2d::geometry::{BroadPhase, ContactManifold, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use std::time::Duration;

#[derive(Default)]
pub(super) struct PipelineStorage {
    pub(super) island_manager: IslandManager,
    pub(super) impulse_joint_set: ImpulseJointSet,
    pub(super) multibody_joint_set: MultibodyJointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
}

impl PipelineStorage {
    pub(super) fn contacts<'a>(
        &'a self,
        colliders: &'a ColliderStorage,
    ) -> impl Iterator<Item = Contact<'_>> + '_ {
        self.narrow_phase.contact_pairs().map(|c| {
            let user_data1 = colliders
                .get(c.collider1)
                .expect("internal error: missing rapier collider1")
                .user_data;
            let user_data2 = colliders
                .get(c.collider2)
                .expect("internal error: missing rapier collider2")
                .user_data;
            Contact {
                entity1_id: UserData::from(user_data1).entity_id(),
                entity2_id: UserData::from(user_data2).entity_id(),
                manifolds: &c.manifolds,
            }
        })
    }

    pub(super) fn run_pipeline_step(
        &mut self,
        delta: Duration,
        bodies: &mut BodyStorage,
        colliders: &mut ColliderStorage,
        collision_groups: &CollisionGroupStorage,
    ) {
        self.integration_parameters.dt = delta.as_secs_f32();
        self.physics_pipeline.step(
            &Vector2::zeros(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut bodies.container,
            &mut colliders.container,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            collision_groups,
            &(),
        );
    }
}

pub(super) struct Contact<'a> {
    pub(super) entity1_id: usize,
    pub(super) entity2_id: usize,
    pub(super) manifolds: &'a [ContactManifold],
}
