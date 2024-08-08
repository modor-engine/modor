use log::Level;
use modor::{App, FromApp, State, Updater};
use std::marker::PhantomData;

#[modor::test]
fn run_update() {
    let app = App::new::<Root>(Level::Info);
    let mut value = Value::default();
    value
        .updater()
        .integer(10)
        .for_integer(&app, |i| *i += 1)
        .for_string(&app, String::pop)
        .additional_integer(20_u16)
        .apply();
    assert_eq!(value.integer, 31);
    assert_eq!(value.string, "bbc");
    value.updater().apply();
    assert_eq!(value.integer, 31);
    assert_eq!(value.string, "");
}

#[derive(FromApp, State)]
struct Root;

#[derive(Updater)]
struct Value {
    #[updater(field, for_field = "default")]
    integer: u8,
    #[updater(for_field = "|updated, _| updated.string.replace('a', \"b\")")]
    string: String,
    #[updater(inner_type, field)]
    additional_integer: PhantomData<u16>,
}

impl Default for Value {
    fn default() -> Self {
        Self {
            integer: 5,
            string: "abcd".into(),
            additional_integer: PhantomData,
        }
    }
}

impl ValueUpdater<'_> {
    #[allow(clippy::cast_possible_truncation)]
    fn apply(self) {
        let mut is_updated = false;
        is_updated |= modor::update_field(&mut self.updated.integer, self.integer);
        is_updated |= modor::update_field(&mut self.updated.string, self.string);
        if let Some(additional_integer) = self.additional_integer {
            self.updated.integer += additional_integer as u8;
        }
        if !is_updated {
            self.updated.string.clear();
        }
    }
}
