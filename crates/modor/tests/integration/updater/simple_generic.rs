use log::Level;
use modor::{App, FromApp, State, Updater};

#[modor::test]
fn run_update() {
    let app = App::new::<Root>(Level::Info);
    let mut value = Value::<2, &str>::default();
    value
        .updater()
        .integer(10)
        .dynamic("abc")
        .for_array(&app, |array| array[0] = 10)
        .apply();
    assert_eq!(value.integer, 10);
    assert_eq!(value.dynamic, "abc");
    assert_eq!(value.array[0], 10);
    assert_eq!(value.array[1], 0);
}

#[derive(FromApp, State)]
struct Root;

#[derive(Updater)]
struct Value<const N: usize, T: 'static> {
    #[updater(field)]
    integer: u8,
    #[updater(field)]
    dynamic: T,
    #[updater(for_field = "default")]
    array: [u16; N],
}

impl<const N: usize, T> Default for Value<N, T>
where
    T: 'static + Default,
{
    fn default() -> Self {
        Self {
            integer: 5,
            dynamic: T::default(),
            array: [0; N],
        }
    }
}

impl<const N: usize, T> ValueUpdater<'_, N, T>
where
    T: 'static + Default + PartialEq,
{
    fn apply(self) {
        modor::update_field(&mut self.updated.integer, self.integer, &mut false);
        modor::update_field(&mut self.updated.dynamic, self.dynamic, &mut false);
        modor::update_field(&mut self.updated.array, self.array, &mut false);
    }
}
