use crate::storages_2d::bodies::{BodyState, BodyStorage};
use crate::storages_2d::core::{PhysicsEntity2D, PhysicsEntity2DTuple};
use crate::storages_2d::pipeline::PipelineStorage;
use crate::utils::UserData;
use modor::Query;
use rapier2d::geometry::{ActiveCollisionTypes, Collider, ColliderHandle, ColliderSet};
use rapier2d::prelude::ActiveHooks;

#[derive(Default)]
pub(super) struct ColliderStorage {
    pub(super) container: ColliderSet,
}

impl ColliderStorage {
    pub(super) fn get(&self, handle: ColliderHandle) -> Option<&Collider> {
        self.container.get(handle)
    }

    pub(super) fn get_mut(&mut self, handle: ColliderHandle) -> Option<&mut Collider> {
        self.container.get_mut(handle)
    }

    pub(super) fn delete_outdated_handles(&mut self, entity: &mut PhysicsEntity2D<'_>) {
        if let Some(collider) = &mut entity.collider {
            if let Some(handle) = collider.handle {
                if let Some(rapier_collider) = self.container.get_mut(handle) {
                    if UserData::from(rapier_collider.user_data).entity_id() != entity.entity.id() {
                        collider.handle = None;
                    }
                } else {
                    collider.handle = None;
                }
            }
        }
    }

    pub(super) fn delete_outdated(
        &mut self,
        entities: &mut Query<'_, PhysicsEntity2DTuple<'_>>,
        bodies: &mut BodyStorage,
        pipeline: &mut PipelineStorage,
    ) {
        self.enable_deletion_flags_for_all_colliders();
        self.disable_deletion_flag_for_existing_colliders(entities);
        self.delete_colliders_with_enabled_deletion_flag(bodies, pipeline);
    }

    pub(super) fn create(
        &mut self,
        entity: &mut PhysicsEntity2D<'_>,
        body_state: BodyState,
        bodies: &mut BodyStorage,
        pipeline: &mut PipelineStorage,
    ) {
        if let Some(collider) = &mut entity.collider {
            // Remove existing collider in case rigid body has just been created,
            // because collider must be created after rigid body in order to be attached.
            if let BodyState::Created(_) = body_state {
                if let Some(collider_handle) = collider.handle {
                    self.container.remove(
                        collider_handle,
                        &mut pipeline.island_manager,
                        &mut bodies.container,
                        true,
                    );
                }
                collider.handle = None;
            }
            // Create the collider if not already existing.
            if collider.handle.is_none() {
                let builder = collider
                    .collider_builder(*entity.transform.size)
                    .user_data(UserData::new(entity.entity.id()).into())
                    .active_collision_types(ActiveCollisionTypes::all())
                    .active_hooks(
                        ActiveHooks::FILTER_CONTACT_PAIRS | ActiveHooks::FILTER_INTERSECTION_PAIR,
                    );
                collider.handle = Some(if let Some(body_handle) = body_state.handle() {
                    self.container
                        .insert_with_parent(builder, body_handle, &mut bodies.container)
                } else {
                    let builder = entity.transform.updated_collider_builder(builder);
                    self.container.insert(builder)
                });
            }
        }
    }

    fn enable_deletion_flags_for_all_colliders(&mut self) {
        for (_, collider) in self.container.iter_mut() {
            collider.user_data = UserData::from(collider.user_data)
                .with_deletion_flag(true)
                .into();
        }
    }

    fn disable_deletion_flag_for_existing_colliders(
        &mut self,
        entities: &mut Query<'_, PhysicsEntity2DTuple<'_>>,
    ) {
        for entity in entities.iter_mut() {
            let entity = PhysicsEntity2D::from(entity);
            if let Some(Some(handle)) = entity.collider.map(|d| d.handle) {
                let collider = &mut self.container[handle];
                collider.user_data = UserData::from(collider.user_data)
                    .with_deletion_flag(false)
                    .into();
            }
        }
    }

    fn delete_colliders_with_enabled_deletion_flag(
        &mut self,
        bodies: &mut BodyStorage,
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
                &mut bodies.container,
                true,
            );
        }
    }
}
