use crate::entities::module::internal::{
    UpdateAbsolutePositionsAction, UpdateAbsoluteScalesAction, UpdatePositionsAction,
    UpdateVelocitiesAction,
};
use crate::{Acceleration, DeltaTime, Position, Scale, Velocity};
use modor::{Built, Entity, EntityBuilder, Query, Single, With};
use std::marker::PhantomData;

const DEFAULT_POSITION: Position = Position::xyz(0., 0., 0.);
const DEFAULT_SCALE: Scale = Scale::xyz(1., 1., 1.);

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
        entities_with_scale: Query<'_, Entity<'_>, (With<Position>, With<Scale>)>,
        mut scales: Query<'_, &mut Scale, With<Position>>,
    ) {
        let entities = Self::sorted_by_depth(entities_with_scale.iter());
        for entity in entities {
            let (result, parent_result) = scales.get_with_first_parent_mut(entity.id());
            let parent_scale = parent_result.as_deref().unwrap_or(&DEFAULT_SCALE);
            if let Some(scale) = result {
                scale.update_abs(parent_scale);
            }
        }
    }

    #[run_as(UpdateAbsolutePositionsAction)]
    fn update_absolute_positions(
        entities_with_position: Query<'_, Entity<'_>, With<Position>>,
        mut components: Query<'_, (&mut Position, Option<&mut Scale>)>,
    ) {
        let entities = Self::sorted_by_depth(entities_with_position.iter());
        for entity in entities {
            let (result, parent_result) = components.get_with_first_parent_mut(entity.id());
            let (parent_position, parent_scale) = parent_result
                .map_or((&DEFAULT_POSITION, &DEFAULT_SCALE), |(p, s)| {
                    (p, s.map_or(&DEFAULT_SCALE, |a| a))
                });
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
    use crate::UpdateDeltaTimeAction;

    #[action(UpdateDeltaTimeAction)]
    pub struct UpdateVelocitiesAction;

    #[action(UpdateVelocitiesAction)]
    pub struct UpdatePositionsAction;

    #[action(UpdateDeltaTimeAction)]
    pub struct UpdateAbsoluteScalesAction;

    #[action(UpdatePositionsAction, UpdateAbsoluteScalesAction)]
    pub struct UpdateAbsolutePositionsAction;
}

#[cfg(test)]
mod physics_module_tests {
    use crate::{Acceleration, DeltaTime, PhysicsModule, Position, Scale, Velocity};
    use approx::assert_abs_diff_eq;
    use modor::testing::TestApp;
    use modor::{App, EntityBuilder};

    struct TestEntity;

    #[entity]
    impl TestEntity {}

    #[test]
    fn build() {
        let app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
        app.assert_singleton::<PhysicsModule>().exists();
        app.assert_singleton::<DeltaTime>().exists();
    }

    #[test]
    fn update_velocity_and_position() {
        let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
        let entity_id = app.create_entity(
            EntityBuilder::new(TestEntity)
                .with(Position::xyz(1., 2., 3.))
                .with(Velocity::xyz(4., 5., 6.))
                .with(Acceleration::xyz(7., 8., 9.)),
        );
        app.update();
        let mut delta_time = 0.;
        app.assert_singleton::<DeltaTime>()
            .has::<DeltaTime, _>(|d| delta_time = d.get().as_secs_f32());
        app.assert_entity(entity_id)
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.x, 7.))
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.y, 8.))
            .has::<Acceleration, _>(|a| assert_abs_diff_eq!(a.z, 9.))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.x, 7.0_f32.mul_add(delta_time, 4.)))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.y, 8.0_f32.mul_add(delta_time, 5.)))
            .has::<Velocity, _>(|v| assert_abs_diff_eq!(v.z, 9.0_f32.mul_add(delta_time, 6.)))
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.x, 7.0_f32.mul_add(delta_time, 4.).mul_add(delta_time, 1.));
            })
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.y, 8.0_f32.mul_add(delta_time, 5.).mul_add(delta_time, 2.));
            })
            .has::<Position, _>(|p| {
                assert_abs_diff_eq!(p.z, 9.0_f32.mul_add(delta_time, 6.).mul_add(delta_time, 3.));
            });
    }

    #[test]
    fn update_absolute_position() {
        let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
        let entity1_id =
            app.create_entity(EntityBuilder::new(TestEntity).with(Position::xyz(1., 2., 3.)));
        let entity2_id = app.create_child(entity1_id, EntityBuilder::new(TestEntity));
        let entity3_id = app.create_child(
            entity2_id,
            EntityBuilder::new(TestEntity)
                .with(Position::xyz(4., 5., 6.))
                .with(Scale::xyz(0.1, 0.2, 0.5)),
        );
        let entity4_id = app.create_child(
            entity3_id,
            EntityBuilder::new(TestEntity).with(Position::xyz(7., 8., 9.)),
        );
        app.update();
        app.assert_entity(entity1_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 1.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 2.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 3.));
        app.assert_entity(entity3_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 5.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 7.))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 9.));
        app.assert_entity(entity4_id)
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().x, 5.7))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().y, 8.6))
            .has::<Position, _>(|p| assert_abs_diff_eq!(p.abs().z, 13.5));
    }

    #[test]
    fn update_absolute_scale() {
        let mut app: TestApp = App::new().with_entity(PhysicsModule::build()).into();
        let entity1_id = app.create_entity(
            EntityBuilder::new(TestEntity)
                .with(Position::xyz(1., 2., 3.))
                .with(Scale::xyz(1., 2., 3.)),
        );
        let entity2_id = app.create_child(entity1_id, EntityBuilder::new(TestEntity));
        let entity3_id = app.create_child(
            entity2_id,
            EntityBuilder::new(TestEntity)
                .with(Position::xyz(40., 50., 60.))
                .with(Scale::xyz(0.1, 0.2, 0.5)),
        );
        let entity4_id = app.create_child(
            entity3_id,
            EntityBuilder::new(TestEntity).with(Scale::xyz(4., 3., 2.)),
        );
        let entity5_id = app.create_child(
            entity4_id,
            EntityBuilder::new(TestEntity)
                .with(Position::xyz(70., 80., 90.))
                .with(Scale::xyz(0.5, 0.2, 0.1)),
        );
        app.update();
        app.assert_entity(entity1_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 1.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 2.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 3.));
        app.assert_entity(entity3_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 0.1))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 0.4))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 1.5));
        app.assert_entity(entity4_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 4.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 3.))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 2.));
        app.assert_entity(entity5_id)
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().x, 0.05))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().y, 0.08))
            .has::<Scale, _>(|s| assert_abs_diff_eq!(s.abs().z, 0.15));
    }
}
