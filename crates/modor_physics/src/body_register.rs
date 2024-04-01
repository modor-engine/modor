use crate::collision_group::CollisionGroupRegister;
use crate::{Body2DIndex, Collision2D, Delta};
use fxhash::FxHashMap;
use modor::{Context, NoVisit, Node, RootNode};
use modor_internal::index::{Index, IndexPool};
use modor_math::Vec2;
use rapier2d::dynamics::{
    CCDSolver, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    RigidBodyHandle, RigidBodySet,
};
use rapier2d::geometry::{BroadPhase, Collider, ColliderHandle, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::PhysicsPipeline;
use rapier2d::prelude::RigidBody;
use std::sync::Arc;

/// The type responsible to register and update [`Body2D`](crate::Body2D)s.
#[derive(Default, RootNode, NoVisit)]
pub struct Body2DRegister {
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    island_manager: IslandManager,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    integration_parameters: IntegrationParameters,
    #[allow(clippy::struct_field_names)]
    physics_pipeline: PhysicsPipeline,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    body_ids: Arc<IndexPool>,
    body_details: Vec<Option<BodyDetails>>,
    collider_body_ids: FxHashMap<ColliderHandle, usize>,
}

impl Node for Body2DRegister {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        for index in self.body_ids.take_deleted_indexes() {
            self.delete_body(index);
        }
        self.integration_parameters.dt = ctx.root::<Delta>().duration.as_secs_f32();
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
            ctx.root::<CollisionGroupRegister>(),
            &(),
        );
        self.reset_collisions();
        self.register_collisions();
    }
}

impl Body2DRegister {
    /// Returns the transform properties of a [`Body2D`](crate::Body2D) with a given `index`.
    pub fn transform(&self, index: &Body2DIndex) -> Transform2D {
        let details = self.body_details[index.as_usize()]
            .as_ref()
            .expect("internal error: missing body");
        let rigid_body = &self.rigid_bodies[details.rigid_body];
        Transform2D {
            position: Vec2::new(rigid_body.translation().x, rigid_body.translation().y),
            size: details.size,
            rotation: rigid_body.rotation().angle(),
        }
    }

    pub(crate) fn rigid_body_mut(&mut self, index: &Index) -> &mut RigidBody {
        let details = self.body_details[index.value()]
            .as_mut()
            .expect("internal error: missing body");
        &mut self.rigid_bodies[details.rigid_body]
    }

    pub(crate) fn collider_mut(&mut self, index: &Index) -> &mut Collider {
        let details = self.body_details[index.value()]
            .as_mut()
            .expect("internal error: missing body");
        &mut self.colliders[details.collider]
    }

    pub(crate) fn set_size(&mut self, index: &Index, size: Vec2) {
        self.body_details[index.value()]
            .as_mut()
            .expect("internal error: missing body")
            .size = size;
    }

    pub(crate) fn collisions(&mut self, index: &Index) -> Vec<Collision2D> {
        self.body_details[index.value()]
            .as_mut()
            .expect("internal error: missing body")
            .collisions
            .clone()
    }

    pub(crate) fn register_body(
        &mut self,
        rigid_body: RigidBody,
        collider: Collider,
        size: Vec2,
    ) -> Index {
        let index = self.body_ids.generate();
        for _ in self.body_details.len()..=index.value() {
            self.body_details.push(None);
        }
        let rigid_body = self.rigid_bodies.insert(rigid_body);
        let collider =
            self.colliders
                .insert_with_parent(collider, rigid_body, &mut self.rigid_bodies);
        self.body_details[index.value()] = Some(BodyDetails {
            rigid_body,
            collider,
            collisions: vec![],
            size,
        });
        self.collider_body_ids.insert(collider, index.value());
        index
    }

    fn delete_body(&mut self, index: usize) {
        let details = self.body_details[index]
            .as_mut()
            .expect("internal error: missing body");
        self.rigid_bodies.remove(
            details.rigid_body,
            &mut self.island_manager,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            true,
        );
        self.body_details[index] = None;
    }

    fn reset_collisions(&mut self) {
        for body in self.body_details.iter_mut().flatten() {
            body.collisions.clear();
        }
    }

    fn register_collisions(&mut self) {
        for pair in self.narrow_phase.contact_pairs() {
            let body1_index = self.collider_body_ids[&pair.collider1];
            let body2_index = self.collider_body_ids[&pair.collider2];
            let collider1 = &self.colliders[pair.collider1];
            let collider2 = &self.colliders[pair.collider2];
            let group1_index = collider1.user_data as usize;
            let group2_index = collider2.user_data as usize;
            for manifold in &pair.manifolds {
                if manifold.points.iter().all(|p| p.dist > 0.) {
                    continue;
                }
                let body1 = self.body_details[body1_index]
                    .as_mut()
                    .expect("internal error: missing body");
                body1.collisions.push(Collision2D::new(
                    false,
                    body2_index,
                    group2_index,
                    collider1,
                    manifold,
                ));
                let body2 = self.body_details[body2_index]
                    .as_mut()
                    .expect("internal error: missing body");
                body2.collisions.push(Collision2D::new(
                    true,
                    body1_index,
                    group1_index,
                    collider2,
                    manifold,
                ));
            }
        }
    }
}

/// The transform properties of a 2D object.
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    /// Position of the object in world units.
    pub position: Vec2,
    /// Size of the object in world units.
    pub size: Vec2,
    /// Rotation of the object in radians.
    pub rotation: f32,
}

struct BodyDetails {
    rigid_body: RigidBodyHandle,
    collider: ColliderHandle,
    collisions: Vec<Collision2D>,
    size: Vec2,
}
