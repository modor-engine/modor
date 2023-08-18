use crate::storages_2d::colliders::ColliderStorage;
use crate::storages_2d::core::PhysicsEntity2D;
use crate::storages_2d::pipeline::PipelineStorage;
use crate::utils::UserData;
use modor::{Custom, Query};
use rapier2d::dynamics::{
    RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet, RigidBodyType,
};

#[derive(Default)]
pub(super) struct BodyStorage {
    pub(super) container: RigidBodySet,
}

impl BodyStorage {
    pub(super) fn get(&self, handle: RigidBodyHandle) -> Option<&RigidBody> {
        self.container.get(handle)
    }

    pub(super) fn get_mut(&mut self, handle: RigidBodyHandle) -> Option<&mut RigidBody> {
        self.container.get_mut(handle)
    }

    pub(super) fn delete_outdated_handles(&mut self, entity: &mut PhysicsEntity2D<'_>) {
        if let Some(dynamics) = &mut entity.dynamics {
            if let Some(handle) = dynamics.handle {
                if let Some(body) = self.container.get_mut(handle) {
                    if UserData::from(body.user_data).entity_id() != entity.entity.id() {
                        dynamics.handle = None;
                    }
                } else {
                    dynamics.handle = None;
                }
            }
        }
    }

    pub(super) fn delete_outdated(
        &mut self,
        entities: &mut Query<'_, Custom<PhysicsEntity2D<'_>>>,
        colliders: &mut ColliderStorage,
        pipeline: &mut PipelineStorage,
    ) {
        self.enable_deletion_flags_for_all_bodies();
        self.disable_deletion_flag_for_existing_bodies(entities);
        self.delete_bodies_with_enabled_deletion_flag(colliders, pipeline);
    }

    pub(super) fn create(&mut self, entity: &mut PhysicsEntity2D<'_>) -> BodyState {
        if entity.relative_transform.is_some() {
            return BodyState::Missing;
        }
        if let Some(dynamics) = &mut entity.dynamics {
            if let Some(handle) = dynamics.handle {
                BodyState::Existing(handle)
            } else {
                let builder = RigidBodyBuilder::new(RigidBodyType::Dynamic)
                    .can_sleep(false)
                    .user_data(UserData::new(entity.entity.id()).into());
                let builder = entity.transform.updated_body_builder(builder);
                let builder = dynamics.updated_body_builder(builder);
                let handle = self.container.insert(builder);
                dynamics.handle = Some(handle);
                BodyState::Created(handle)
            }
        } else {
            BodyState::Missing
        }
    }

    fn enable_deletion_flags_for_all_bodies(&mut self) {
        for (_, body) in self.container.iter_mut() {
            body.user_data = UserData::from(body.user_data)
                .with_deletion_flag(true)
                .into();
        }
    }

    fn disable_deletion_flag_for_existing_bodies(
        &mut self,
        entities: &mut Query<'_, Custom<PhysicsEntity2D<'_>>>,
    ) {
        for entity in entities.iter_mut() {
            if entity.relative_transform.is_some() {
                continue;
            }
            if let Some(Some(handle)) = entity.dynamics.as_ref().map(|d| d.handle) {
                let body = &mut self.container[handle];
                body.user_data = UserData::from(body.user_data)
                    .with_deletion_flag(false)
                    .into();
            }
        }
    }

    fn delete_bodies_with_enabled_deletion_flag(
        &mut self,
        colliders: &mut ColliderStorage,
        pipeline: &mut PipelineStorage,
    ) {
        let handles_to_delete: Vec<_> = self
            .container
            .iter()
            .filter(|(_, b)| UserData::from(b.user_data).has_deletion_flag())
            .map(|(h, _)| h)
            .collect();
        for handle in handles_to_delete {
            self.container.remove(
                handle,
                &mut pipeline.island_manager,
                &mut colliders.container,
                &mut pipeline.impulse_joint_set,
                &mut pipeline.multibody_joint_set,
                false,
            );
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum BodyState {
    Created(RigidBodyHandle),
    Existing(RigidBodyHandle),
    Missing,
}

impl BodyState {
    pub(super) fn handle(self) -> Option<RigidBodyHandle> {
        match self {
            Self::Created(handle) | Self::Existing(handle) => Some(handle),
            Self::Missing => None,
        }
    }
}
