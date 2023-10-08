use crate::{DeltaTime, Dynamics2D};
use modor::{Query, SingleRef};
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::RigidBody;

#[derive(SingletonComponent, Default)]
pub(crate) struct Pipeline2D {
    bodies: RigidBodySet,
    colliders: ColliderSet,
    island_manager: IslandManager,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
}

#[systems]
impl Pipeline2D {
    pub(crate) fn create_body(&mut self, builder: RigidBodyBuilder) -> RigidBodyHandle {
        self.bodies.insert(builder)
    }

    pub(crate) fn body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.bodies.get_mut(handle)
    }

    pub(crate) fn delete_body(&mut self, handle: RigidBodyHandle) {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            false,
        );
    }

    #[run_after(action(HandleRemoval))]
    fn delete_unsynchronized_bodies(&mut self, dynamics: Query<'_, &Dynamics2D>) {
        let unsynchronized_body_handles: Vec<_> = self
            .bodies
            .iter()
            .filter(|(handle, body)| Self::is_body_unsynchronized(&dynamics, *handle, body))
            .map(|(handle, _)| handle)
            .collect();
        for handle in unsynchronized_body_handles {
            self.delete_body(handle);
        }
    }

    #[run_after_previous_and(component(DeltaTime))]
    fn update(&mut self, delta: SingleRef<'_, '_, DeltaTime>) {
        self.integration_parameters.dt = delta.get().get().as_secs_f32();
        self.physics_pipeline.step(
            &Vector2::zeros(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            None,
            &(),
            &(),
        );
    }

    fn is_body_unsynchronized(
        dynamics: &Query<'_, &Dynamics2D>,
        handle: RigidBodyHandle,
        body: &RigidBody,
    ) -> bool {
        dynamics
            .get(body.user_data as usize)
            .and_then(|d| d.handle)
            .map_or(true, |current_handle| current_handle != handle)
    }
}

#[derive(Action)]
pub(crate) struct HandleRemoval;
