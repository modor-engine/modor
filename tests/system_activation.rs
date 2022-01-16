use modor::testing::TestApp;
use modor::{
    system, Built, Entity, EntityBuilder, EntityMainComponent, EntityRunner, Query, World,
};

struct SignalCounter(usize);

impl EntityMainComponent for SignalCounter {
    type Data = ();

    fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built {
        builder.with_self(Self(0))
    }
}

struct Action;

impl EntityMainComponent for Action {
    type Data = ();

    fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built {
        builder.with_self(Self)
    }

    fn on_update(runner: EntityRunner<'_, Self>) {
        runner
            .run(system!(Self::update_counters))
            .run(system!(Self::destroy));
    }
}

impl Action {
    fn update_counters(mut counters: Query<'_, &mut SignalCounter>) {
        for counter in counters.iter_mut() {
            counter.0 += 1;
        }
    }

    fn destroy(entity: Entity<'_>, mut world: World<'_>) {
        world.delete_entity(entity.id());
    }
}

#[test]
fn keep_system_disabled() {
    let mut app = TestApp::new();
    let counter_id = app.create_entity::<SignalCounter>(());

    app.update();

    app.assert_entity(counter_id)
        .has::<SignalCounter, _>(|c| assert_eq!(c.0, 0));
}

#[test]
fn activate_system() {
    let mut app = TestApp::new();
    let counter_id = app.create_entity::<SignalCounter>(());
    app.create_entity::<Action>(());
    app.create_entity::<Action>(());

    app.update();

    app.assert_entity(counter_id)
        .has::<SignalCounter, _>(|c| assert_eq!(c.0, 1));
}

#[test]
fn deactivate_system() {
    let mut app = TestApp::new();
    let counter_id = app.create_entity::<SignalCounter>(());
    app.create_entity::<Action>(());
    app.create_entity::<Action>(());

    app.update();
    app.update();

    app.assert_entity(counter_id)
        .has::<SignalCounter, _>(|c| assert_eq!(c.0, 1));
}

#[test]
fn reactivate_system() {
    let mut app = TestApp::new();
    let counter_id = app.create_entity::<SignalCounter>(());
    app.create_entity::<Action>(());
    app.create_entity::<Action>(());

    app.update();
    app.update();
    app.create_entity::<Action>(());
    app.create_entity::<Action>(());
    app.update();

    app.assert_entity(counter_id)
        .has::<SignalCounter, _>(|c| assert_eq!(c.0, 2));
}
