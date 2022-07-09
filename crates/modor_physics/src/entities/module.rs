use crate::components::dynamic_body::DynamicBody;
use crate::components::relative_transform::RelativeTransform;
use crate::components::transform::Transform;
use crate::entities::module::internal::{
    UpdateDynamicBodiesAction, UpdateTransformsFromRelativeAction,
};
use crate::{DeltaTime, ROOT_TRANSFORM};
use modor::{Built, Entity, EntityBuilder, Query, Single, With};
use std::marker::PhantomData;

/// The main entity of the physics module.
///
/// # Modor
///
/// - **Type**: singleton entity
/// - **Lifetime**: custom (same as parent entity)
///
/// # Examples
///
/// ```rust
/// # use std::f32::consts::PI;
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_math::{Quat, Vec3};
/// # use modor_physics::{Transform, PhysicsModule, DynamicBody, RelativeTransform};
/// #
/// let mut app = App::new()
///     .with_entity(PhysicsModule::build())
///     .with_entity(Object::build());
/// loop {
///     app.update();
///     # break;
/// }
///
/// struct Object;
///
/// #[entity]
/// impl Object {
///     fn build() -> impl Built<Self> {
///         EntityBuilder::new(Self)
///             .with(
///                 Transform::new()
///                     .with_position(Vec3::xy(0.2, 0.3))
///                     .with_size(Vec3::xyz(0.25, 0.5, 1.))
///                     .with_rotation(Quat::from_z(20_f32.to_radians()))
///             )
///             .with(RelativeTransform::new().with_rotation(Quat::from_z(PI / 2.)))
///             .with(DynamicBody::new().with_velocity(Vec3::xy(-0.01, 0.02)))
///     }
/// }
/// ```
pub struct PhysicsModule(PhantomData<()>);

#[singleton]
impl PhysicsModule {
    /// Builds the module.
    pub fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(PhantomData)).with_child(DeltaTime::build())
    }

    #[run_as(UpdateDynamicBodiesAction)]
    fn update_dynamic_bodies(
        mut bodies: Query<
            '_,
            (
                &mut DynamicBody,
                &mut Transform,
                Option<&mut RelativeTransform>,
            ),
        >,
        delta_time: Single<'_, DeltaTime>,
    ) {
        for (dynamic, transform, relative) in bodies.iter_mut() {
            dynamic.update(transform, relative, &*delta_time);
        }
    }

    #[run_as(UpdateTransformsFromRelativeAction)]
    fn update_transforms_from_relative(
        entities: Query<'_, Entity<'_>, (With<Transform>, With<RelativeTransform>)>,
        mut components: Query<'_, (&mut Transform, Option<&RelativeTransform>)>,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter()) {
            match components.get_with_first_parent_mut(entity.id()) {
                (Some((transform, Some(relative))), Some((parent, _))) => {
                    transform.update(relative, parent);
                }
                (Some((transform, Some(relative))), None) => {
                    transform.update(relative, &ROOT_TRANSFORM);
                }
                _ => unreachable!("internal error: unreachable position update case"),
            }
        }
    }

    #[run_as(UpdatePhysicsAction)]
    fn finish_update() {}

    fn entities_sorted_by_depth<'a, I>(entities: I) -> Vec<Entity<'a>>
    where
        I: Iterator<Item = Entity<'a>>,
    {
        let mut entities: Vec<_> = entities.collect();
        entities.sort_unstable_by_key(|e| e.depth());
        entities
    }
}

/// An action done when the positions and sizes have been updated.
#[action(UpdateTransformsFromRelativeAction)]
pub struct UpdatePhysicsAction;

mod internal {
    #[action]
    pub struct UpdateDynamicBodiesAction;

    #[action(UpdateDynamicBodiesAction)]
    pub struct UpdateTransformsFromRelativeAction;
}
