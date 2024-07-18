use log::Level;
use modor::{App, FromApp};
use modor_derive::Singleton;

#[modor::test]
fn create_singleton_with_named_fields() {
    let mut app = App::new(Level::Info);
    assert_eq!(app.get_mut::<NamedFields>().field1.0, 10);
    assert_eq!(app.get_mut::<NamedFields>().field2, 0);
}

#[modor::test]
fn create_singleton_with_unnamed_fields() {
    let mut app = App::new(Level::Info);
    assert_eq!(app.get_mut::<UnnamedFields>().0 .0, 10);
    assert_eq!(app.get_mut::<UnnamedFields>().1, 0);
}

#[derive(FromApp, Singleton)]
struct NamedFields {
    field1: Integer,
    field2: u8,
}

#[derive(FromApp, Singleton)]
struct UnnamedFields(Integer, u8);

#[derive(FromApp, Singleton)]
struct Unit;

#[derive(Singleton)]
struct Integer(u32);

impl FromApp for Integer {
    fn from_app(app: &mut App) -> Self {
        Self(app.get_mut::<DefaultInteger>().0)
    }
}

#[derive(Singleton)]
struct DefaultInteger(u32);

impl Default for DefaultInteger {
    fn default() -> Self {
        Self(10)
    }
}
