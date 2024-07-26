use log::Level;
use modor::{App, FromApp};
use modor_derive::State;

#[modor::test]
fn create_struct_with_named_fields() {
    let mut app = App::new::<Root>(Level::Info);
    let value = NamedFields::from_app(&mut app);
    assert_eq!(value.field1.0, 10);
    assert_eq!(value.field2, 0);
}

#[modor::test]
fn create_struct_with_unnamed_fields() {
    let mut app = App::new::<Root>(Level::Info);
    let value = UnnamedFields::from_app(&mut app);
    assert_eq!(value.0 .0, 10);
    assert_eq!(value.1, 0);
}

#[modor::test]
fn create_with_callback() {
    let mut app = App::new::<Root>(Level::Info);
    let value = NamedFields::from_app_with(&mut app, |value, _| value.field2 += 1);
    assert_eq!(value.field2, 1);
}

#[derive(Default, State)]
struct Root;

#[derive(FromApp)]
struct NamedFields {
    field1: Integer,
    field2: u8,
}

#[derive(FromApp)]
struct UnnamedFields(Integer, u8);

#[derive(FromApp)]
struct Unit;

struct Integer(u32);

impl FromApp for Integer {
    fn from_app(app: &mut App) -> Self {
        Self(app.get_mut::<DefaultInteger>().0)
    }
}

#[derive(State)]
struct DefaultInteger(u32);

impl Default for DefaultInteger {
    fn default() -> Self {
        Self(10)
    }
}
