use crate::entities::module::internal::UpdateAbsoluteRotationsFromRelativePositionsAction;
use crate::{
    Acceleration, DeltaTime, Position, RelativeAcceleration, RelativePosition, RelativeRotation,
    RelativeSize, RelativeVelocity, Rotation, Size, Velocity,
};
use internal::{
    UpdateAbsolutePositionsFromRelativePositionsAction,
    UpdateAbsolutePositionsFromVelocitiesAction, UpdateAbsoluteSizesAction,
    UpdateAbsoluteVelocitiesAction, UpdateRelativePositionsAction, UpdateRelativeVelocitiesAction,
};
use modor::{Built, Entity, EntityBuilder, Query, Single, With};
use std::marker::PhantomData;

const ROOT_POSITION: Position = Position::ZERO;
const ROOT_SIZE: Size = Size::ONE;
const ROOT_ROTATION: Rotation = Rotation::ZERO;

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
/// # use modor::{entity, App, Built, EntityBuilder};
/// # use modor_physics::{Acceleration, PhysicsModule, Position, Size, Shape, Velocity};
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
///             .with(Position::xy(0.2, 0.3))
///             .with(Velocity::xy(-0.01, 0.02))
///             .with(Acceleration::xy(0.5, -0.1))
///             .with(Size::xy(0.25, 0.5))
///             .with(Shape::Rectangle2D)
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

    #[run_as(UpdateRelativeVelocitiesAction)]
    fn update_relative_velocities(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut RelativeVelocity, &RelativeAcceleration)>,
    ) {
        for (velocity, acceleration) in components.iter_mut() {
            velocity.update(*acceleration, delta_time.get());
        }
    }

    #[run_as(UpdateRelativePositionsAction)]
    fn update_relative_positions(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut RelativePosition, &RelativeVelocity)>,
    ) {
        for (position, velocity) in components.iter_mut() {
            position.update(*velocity, delta_time.get());
        }
    }

    #[run_as(UpdateAbsoluteVelocitiesAction)]
    fn update_absolute_velocities(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Velocity, &Acceleration)>,
    ) {
        for (velocity, acceleration) in components.iter_mut() {
            velocity.update(*acceleration, delta_time.get());
        }
    }

    #[run_as(UpdateAbsolutePositionsFromVelocitiesAction)]
    fn update_absolute_positions_from_velocities(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Position, &Velocity)>,
    ) {
        for (position, velocity) in components.iter_mut() {
            position.update_with_velocity(*velocity, delta_time.get());
        }
    }

    #[run_as(UpdateAbsoluteSizesAction)]
    fn update_absolute_sizes(
        entities: Query<'_, Entity<'_>, (With<Size>, With<RelativeSize>)>,
        mut components: Query<'_, (&mut Size, Option<&RelativeSize>)>,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter()) {
            match components.get_with_first_parent_mut(entity.id()) {
                (Some((size, Some(relative_size))), Some((parent_size, _))) => {
                    size.update_with_relative(*relative_size, *parent_size);
                }
                (Some((size, Some(relative_size))), None) => {
                    size.update_with_relative(*relative_size, ROOT_SIZE);
                }
                _ => unreachable!("internal error: unreachable size update case"),
            }
        }
    }

    #[run_as(UpdateAbsoluteRotationsFromRelativePositionsAction)]
    fn update_absolute_rotations_from_relative_rotations(
        entities: Query<'_, Entity<'_>, (With<Position>, With<Rotation>, With<RelativeRotation>)>,
        mut components: Query<
            '_,
            (Option<&mut Rotation>, Option<&RelativeRotation>),
            With<Position>,
        >,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter()) {
            match components.get_with_first_parent_mut(entity.id()) {
                (Some((Some(rotation), Some(relative_rotation))), Some((parent_rotation, _))) => {
                    rotation.update_with_relative(
                        *relative_rotation,
                        parent_rotation.copied().unwrap_or(Rotation::ZERO),
                    )
                }
                (Some((Some(rotation), Some(relative_rotation))), None) => {
                    rotation.update_with_relative(*relative_rotation, ROOT_ROTATION);
                }
                _ => unreachable!("internal error: unreachable position update case"),
            }
        }
    }

    #[run_as(UpdateAbsolutePositionsFromRelativePositionsAction)]
    fn update_absolute_positions_from_relative_positions(
        entities: Query<'_, Entity<'_>, (With<Position>, With<RelativePosition>)>,
        mut components: Query<
            '_,
            (
                &mut Position,
                &Size,
                Option<&Rotation>,
                Option<&RelativePosition>,
            ),
        >,
    ) {
        for entity in Self::entities_sorted_by_depth(entities.iter()) {
            match components.get_with_first_parent_mut(entity.id()) {
                (
                    Some((position, _, _, Some(relative_position))),
                    Some((parent_position, parent_size, parent_rotation, _)),
                ) => position.update_with_relative(
                    *relative_position,
                    *parent_position,
                    *parent_size,
                    parent_rotation.copied().unwrap_or(Rotation::ZERO),
                ),
                (Some((position, _, _, Some(relative_position))), None) => {
                    position.update_with_relative(
                        *relative_position,
                        ROOT_POSITION,
                        ROOT_SIZE,
                        ROOT_ROTATION,
                    );
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
#[action(UpdateAbsolutePositionsFromRelativePositionsAction)]
pub struct UpdatePhysicsAction;

mod internal {
    #[action]
    pub struct UpdateRelativeVelocitiesAction;

    #[action(UpdateRelativeVelocitiesAction)]
    pub struct UpdateRelativePositionsAction;

    #[action]
    pub struct UpdateAbsoluteVelocitiesAction;

    #[action(UpdateAbsoluteVelocitiesAction)]
    pub struct UpdateAbsolutePositionsFromVelocitiesAction;

    #[action]
    pub struct UpdateAbsoluteSizesAction;

    #[action()]
    pub struct UpdateAbsoluteRotationsFromRelativePositionsAction;

    #[action(
        UpdateAbsolutePositionsFromVelocitiesAction,
        UpdateRelativePositionsAction,
        UpdateAbsoluteSizesAction,
        UpdateAbsoluteRotationsFromRelativePositionsAction
    )]
    pub struct UpdateAbsolutePositionsFromRelativePositionsAction;
}
