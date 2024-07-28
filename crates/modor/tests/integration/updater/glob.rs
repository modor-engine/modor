use log::Level;
use modor::{App, FromApp, Glob, State};
use modor_derive::{GlobUpdater, Global};
use std::marker::PhantomData;

#[modor::test]
fn run_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = Glob::<Value>::from_app(&mut app);
    glob.updater()
        .integer(10)
        .for_integer(&app, |i| *i += 1)
        .for_string(&app, String::pop)
        .additional_integer(20_u16)
        .apply(&mut app);
    let value = glob.get(&app);
    assert_eq!(value.integer, 31);
    assert_eq!(value.string, "bbc");
    glob.updater().apply(&mut app);
    let value = glob.get(&app);
    assert_eq!(value.integer, 31);
    assert_eq!(value.string, "");
}

#[derive(FromApp, State)]
struct Root;

#[derive(Global, GlobUpdater)]
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
    fn apply(self, app: &mut App) {
        let glob = self.glob.get_mut(app);
        let mut is_updated = false;
        modor::update_field(&mut glob.integer, self.integer, &mut is_updated);
        modor::update_field(&mut glob.string, self.string, &mut is_updated);
        if let Some(additional_integer) = self.additional_integer {
            glob.integer += additional_integer as u8;
        }
        if !is_updated {
            glob.string.clear();
        }
    }
}
