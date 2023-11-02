use crate::components::collider::ColliderUserData;
use crate::components::collision_groups::CollisionGroupRegistry;
use crate::components::physics_hook::PhysicsHook;
use crate::{Collider2D, Collision2D, CollisionGroup, DeltaTime, Dynamics2D};
use modor::{ComponentSystems, Query, SingleRef};
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, ColliderBuilder, ColliderHandle, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::{Collider, RigidBody};

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
    #[run_as(action(UnsynchronizedHandleDeletion))]
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

    #[run_as(action(UnsynchronizedHandleDeletion))]
    fn delete_unsynchronized_colliders(&mut self, colliders: Query<'_, &Collider2D>) {
        let unsynchronized_collider_handles: Vec<_> = self
            .colliders
            .iter()
            .filter(|(handle, body)| Self::is_collider_unsynchronized(&colliders, *handle, body))
            .map(|(handle, _)| handle)
            .collect();
        for handle in unsynchronized_collider_handles {
            self.delete_collider(handle);
        }
    }

    #[run_after_previous_and(
        component(DeltaTime),
        component(PhysicsHook),
        action(BodyUpdate),
        action(ColliderUpdate)
    )]
    fn update(
        &mut self,
        delta: SingleRef<'_, '_, DeltaTime>,
        hook: SingleRef<'_, '_, PhysicsHook>,
        mut collider_entities: Query<'_, &mut Collider2D>,
    ) {
        self.integration_parameters.dt = delta.get().get().as_secs_f32();
        self.run_step(hook.get());
        for collider in collider_entities.iter_mut() {
            collider.collisions.clear();
        }
        for pair in self.narrow_phase.contact_pairs() {
            let entity1_id = self.collider_entity_id(pair.collider1);
            let entity2_id = self.collider_entity_id(pair.collider2);
            let rapier_collider_1 = &self.colliders[pair.collider1];
            let rapier_collider_2 = &self.colliders[pair.collider2];
            let (collider1, collider2) = collider_entities.get_both_mut(entity1_id, entity2_id);
            let collider1 = collider1.expect("internal error: collider not found");
            let collider2 = collider2.expect("internal error: collider not found");
            for manifold in &pair.manifolds {
                if manifold.points.iter().all(|p| p.dist > 0.) {
                    continue;
                }
                collider1.collisions.push(Collision2D::new(
                    false,
                    entity2_id,
                    collider2.group_key,
                    rapier_collider_1,
                    manifold,
                ));
                collider2.collisions.push(Collision2D::new(
                    true,
                    entity1_id,
                    collider1.group_key,
                    rapier_collider_2,
                    manifold,
                ));
            }
        }
    }

    pub(crate) fn body_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.bodies.get_mut(handle)
    }

    pub(crate) fn create_body(&mut self, builder: RigidBodyBuilder) -> RigidBodyHandle {
        self.bodies.insert(builder)
    }

    pub(crate) fn collider_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.colliders.get_mut(handle)
    }

    pub(crate) fn create_collider(
        &mut self,
        builder: ColliderBuilder,
        body_handle: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        if let Some(body_handle) = body_handle {
            self.colliders
                .insert_with_parent(builder, body_handle, &mut self.bodies)
        } else {
            self.colliders.insert(builder)
        }
    }

    fn run_step(&mut self, hook: &PhysicsHook) {
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
            hook,
            &(),
        );
    }

    fn collider_entity_id(&self, handle: ColliderHandle) -> usize {
        ColliderUserData::from(self.colliders[handle].user_data).entity_id()
    }

    fn delete_body(&mut self, handle: RigidBodyHandle) {
        self.bodies.remove(
            handle,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            false,
        );
    }

    fn delete_collider(&mut self, handle: ColliderHandle) {
        self.colliders
            .remove(handle, &mut self.island_manager, &mut self.bodies, true);
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

    fn is_collider_unsynchronized(
        collider: &Query<'_, &Collider2D>,
        handle: ColliderHandle,
        rapier_collider: &Collider,
    ) -> bool {
        collider
            .get(ColliderUserData::from(rapier_collider.user_data).entity_id())
            .and_then(|d| d.handle)
            .map_or(true, |current_handle| current_handle != handle)
    }
}

#[derive(Action)]
pub(crate) struct BodyHandleReset;

#[derive(Action)]
pub(crate) struct ColliderHandleRemoval(BodyHandleReset);

#[derive(Action)]
pub(crate) struct UnsynchronizedHandleDeletion(BodyHandleReset, ColliderHandleRemoval);

#[derive(Action)]
pub(crate) struct BodyUpdate(UnsynchronizedHandleDeletion);

#[derive(Action)]
pub(crate) struct ColliderUpdate(
    UnsynchronizedHandleDeletion,
    BodyUpdate,
    <CollisionGroup as ComponentSystems>::Action,
    <CollisionGroupRegistry as ComponentSystems>::Action,
);
