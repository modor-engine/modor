use modor::{App, Built, Changed, Entity, EntityBuilder, Filter, Query, With, World};

#[derive(Component)]
struct TrackedComponent;

#[derive(Component)]
struct Counter(u32);

struct BaseEntity;

#[entity]
impl BaseEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
            .with(TrackedComponent)
            .with(Counter(0))
    }

    #[run_after(entity(StaticEntity), entity(MutatedEntity))]
    fn update(counter: &mut Counter, _: Filter<Changed<TrackedComponent>>) {
        counter.0 += 1;
    }
}

struct StaticEntity;

#[entity]
impl StaticEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }
}

struct MutatedEntity;

#[entity]
impl MutatedEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }

    #[run]
    fn update(_position: &mut TrackedComponent) {}
}

struct UnusedQueryMutatedEntity;

#[entity]
impl UnusedQueryMutatedEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }

    #[run]
    fn update(_query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {}
}

struct ConstQueryMutatedEntity;

#[entity]
impl ConstQueryMutatedEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }

    #[run]
    fn update(query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {
        for _ in query.iter() {}
    }
}

struct MutQueryMutatedEntity;

#[entity]
impl MutQueryMutatedEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }

    #[run]
    fn update(mut query: Query<'_, (&mut TrackedComponent, Filter<With<Self>>)>) {
        for _ in query.iter_mut() {}
    }
}

struct OverwrittenEntity;

#[entity]
impl OverwrittenEntity {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self).inherit_from(BaseEntity::build())
    }

    #[run]
    fn update(entity: Entity<'_>, mut world: World<'_>) {
        world.add_component(entity.id(), TrackedComponent);
    }
}

#[test]
fn filter_by_changed_component() {
    App::new()
        .with_entity(StaticEntity::build())
        .with_entity(MutatedEntity::build())
        .with_entity(UnusedQueryMutatedEntity::build())
        .with_entity(ConstQueryMutatedEntity::build())
        .with_entity(MutQueryMutatedEntity::build())
        .with_entity(OverwrittenEntity::build())
        .updated()
        .updated()
        .assert::<With<StaticEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<MutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .assert::<With<UnusedQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<ConstQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 1)))
        .assert::<With<MutQueryMutatedEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .assert::<With<OverwrittenEntity>>(1, |e| e.has(|c: &Counter| assert_eq!(c.0, 2)))
        .with_entity(StaticEntity::build())
        .updated()
        .updated()
        .assert::<With<StaticEntity>>(2, |e| {
            e.any()
                .has(|c: &Counter| assert_eq!(c.0, 1))
                .has(|c: &Counter| assert_eq!(c.0, 2))
        });
}
