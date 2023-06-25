#![allow(clippy::redundant_closure_for_method_calls)]

use modor::{App, BuiltEntity, EntityBuilder, EntityMut, Not, With};

macro_rules! entity_tester {
    ($component_type:ident, $closure:expr) => {
        #[derive(Default, Component)]
        struct $component_type(bool);

        #[systems]
        impl $component_type {
            #[run]
            #[allow(clippy::redundant_closure_call)]
            fn update(&mut self, mut entity: EntityMut<'_>) {
                let closure: &mut dyn Fn(&mut EntityMut<'_>) = &mut $closure;
                closure(&mut entity);
                self.0 = true;
            }
        }
    };
}

#[modor_test]
fn access_entity() {
    entity_tester!(Tester, |e| assert_eq!(e.entity().depth(), 0));
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|a: &Tester| assert!(a.0)));
}

#[modor_test]
fn access_world() {
    entity_tester!(Tester, |e| e.world().create_root_entity(C1(2)));
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<With<C1>>(2, |e| e);
}

#[modor_test]
fn create_child() {
    entity_tester!(Tester, |e| e.create_child(entity(10)));
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<Not<With<Tester>>>(1, |e| e.has_parent::<With<Tester>>());
}

#[modor_test]
fn delete() {
    entity_tester!(Tester, |e| e.delete());
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<With<Tester>>(0, |e| e);
}

#[modor_test]
fn add_component() {
    entity_tester!(Tester, |e| e.add_component(C3(2)));
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has(|c: &C3| assert_eq!(c.0, 2)));
}

#[modor_test]
fn delete_component() {
    entity_tester!(Tester, |e| e.delete_component::<C2>());
    App::new()
        .with_entity(entity(0).with(Tester::default()))
        .updated()
        .assert::<With<Tester>>(1, |e| e.has::<C1>(|_| ()).has_not::<C2>());
}

fn entity(offset: u32) -> impl BuiltEntity {
    EntityBuilder::new().with(C1(offset)).with(C2(offset + 1))
}

#[derive(Component, NoSystem)]
struct C1(u32);

#[derive(Component, NoSystem)]
struct C2(u32);

#[derive(Component, NoSystem)]
struct C3(u32);
