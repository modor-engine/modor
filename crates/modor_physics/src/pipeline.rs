use crate::collisions::Collision2D;
use crate::physics_hooks::PhysicsHooks;
use crate::user_data::ColliderUserData;
use crate::{Body2DGlob, Delta};
use fxhash::FxHashMap;
use modor::{App, Globals, Node, RootNode, Visit};
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhaseMultiSap, Collider, ColliderHandle, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::RigidBody;

#[derive(Default, RootNode, Visit)]
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
    collisions: FxHashMap<ColliderHandle, Vec<Collision2D>>,
}

impl Node for Pipeline {
    fn on_exit(&mut self, app: &mut App) {
        for (_, body) in app.get_mut::<Globals<Body2DGlob>>().deleted_items() {
            self.rigid_bodies.remove(
                body.rigid_body_handle
                    .expect("internal error: missing body handle"),
                &mut self.island_manager,
                &mut self.colliders,
                &mut self.impulse_joints,
                &mut self.multibody_joints,
                true,
            );
            self.collisions.remove(
                &body
                    .collider_handle
                    .expect("internal error: missing collider handle"),
            );
        }
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
    }
}

impl Pipeline {
    pub(crate) fn rigid_body_mut(&mut self, handle: RigidBodyHandle) -> &mut RigidBody {
        &mut self.rigid_bodies[handle]
    }

    pub(crate) fn collider_mut(&mut self, handle: ColliderHandle) -> &mut Collider {
        &mut self.colliders[handle]
    }

    pub(crate) fn collisions(&mut self, handle: ColliderHandle) -> &[Collision2D] {
        &self.collisions[&handle]
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
        self.collisions.insert(collider_handle, vec![]);
        (rigid_body_handle, collider_handle)
    }

    fn reset_collisions(&mut self) {
        for collisions in self.collisions.values_mut() {
            collisions.clear();
        }
    }

    fn register_collisions(&mut self) {
        for pair in self.narrow_phase.contact_pairs() {
            let collider1 = &self.colliders[pair.collider1];
            let collider2 = &self.colliders[pair.collider2];
            let body1 = ColliderUserData::from(collider1.user_data);
            let body2 = ColliderUserData::from(collider2.user_data);
            for manifold in &pair.manifolds {
                if manifold.points.iter().all(|p| p.dist > 0.) {
                    continue;
                }
                self.collisions
                    .get_mut(&pair.collider1)
                    .expect("internal error: missing body")
                    .push(Collision2D::new(
                        false,
                        body2.body_index(),
                        body2.group_index(),
                        collider1,
                        manifold,
                    ));
                self.collisions
                    .get_mut(&pair.collider2)
                    .expect("internal error: missing body")
                    .push(Collision2D::new(
                        true,
                        body1.body_index(),
                        body1.group_index(),
                        collider2,
                        manifold,
                    ));
            }
        }
    }
}
