use crate::entities::module::internal::{
    UpdateAbsolutePositionsAction, UpdateAbsoluteScalesAction, UpdatePositionsAction,
    UpdateVelocitiesAction,
};
use crate::{Acceleration, DeltaTime, Position, Scale, Velocity};
use modor::{Built, Entity, EntityBuilder, Query, Single, With};
use std::marker::PhantomData;

const ROOT_POSITION: Position = Position::xyz(0., 0., 0.);
const ROOT_SCALE: Scale = Scale::xyz(1., 1., 1.);

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
/// # use modor_physics::{Acceleration, PhysicsModule, Position, Scale, Shape, Velocity};
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
///             .with(Scale::xy(0.25, 0.5))
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

    #[run_as(UpdateVelocitiesAction)]
    fn update_velocities(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Velocity, &Acceleration)>,
    ) {
        for (velocity, acceleration) in components.iter_mut() {
            velocity.update(acceleration, delta_time.get());
        }
    }

    #[run_as(UpdatePositionsAction)]
    fn update_positions(
        delta_time: Single<'_, DeltaTime>,
        mut components: Query<'_, (&mut Position, &Velocity)>,
    ) {
        for (position, velocity) in components.iter_mut() {
            position.update(velocity, delta_time.get());
        }
    }

    #[run_as(UpdateAbsoluteScalesAction)]
    fn update_absolute_scales(
        entities: Query<'_, Entity<'_>, (With<Position>, With<Scale>)>,
        mut scales: Query<'_, &mut Scale, With<Position>>,
    ) {
        let entities = Self::sorted_by_depth(entities.iter());
        for entity in entities {
            let (result, parent_result) = scales.get_with_first_parent_mut(entity.id());
            let parent_scale = parent_result.as_deref().unwrap_or(&ROOT_SCALE);
            if let Some(scale) = result {
                scale.update_abs(parent_scale);
            }
        }
    }

    #[run_as(UpdateAbsolutePositionsAction)]
    fn update_absolute_positions(
        entities: Query<'_, Entity<'_>, (With<Position>, With<Scale>)>,
        mut components: Query<'_, (&mut Position, &mut Scale)>,
    ) {
        let entities = Self::sorted_by_depth(entities.iter());
        for entity in entities {
            let (result, parent_result) = components.get_with_first_parent_mut(entity.id());
            let (parent_position, parent_scale) =
                parent_result.map_or((&ROOT_POSITION, &ROOT_SCALE), |(p, s)| (p, s));
            if let Some((position, _)) = result {
                position.update_abs(parent_position, parent_scale);
            }
        }
    }

    #[run_as(UpdatePhysicsAction)]
    fn finish_update() {}

    fn sorted_by_depth<'a, I>(entities: I) -> Vec<Entity<'a>>
    where
        I: Iterator<Item = Entity<'a>>,
    {
        let mut entities: Vec<_> = entities.collect();
        entities.sort_unstable_by_key(|e| e.depth());
        entities
    }
}

/// An action done when the positions and scales have been updated.
#[action(UpdateAbsolutePositionsAction)]
pub struct UpdatePhysicsAction;

mod internal {
    #[action]
    pub struct UpdateVelocitiesAction;

    #[action(UpdateVelocitiesAction)]
    pub struct UpdatePositionsAction;

    #[action]
    pub struct UpdateAbsoluteScalesAction;

    #[action(UpdatePositionsAction, UpdateAbsoluteScalesAction)]
    pub struct UpdateAbsolutePositionsAction;
}
