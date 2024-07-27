use log::Level;
use modor::{App, FromApp, Glob, GlobUpdater, State};

#[modor::test]
fn run_update() {
    let mut app = App::new::<Root>(Level::Info);
    let glob = Glob::<Value<2, &str>>::from_app(&mut app);
    glob.updater()
        .integer(10)
        .dynamic("abc")
        .for_array(&app, |array| array[0] = 10)
        .apply(&mut app);
    let value = glob.get(&app);
    assert_eq!(value.integer, 10);
    assert_eq!(value.dynamic, "abc");
    assert_eq!(value.array[0], 10);
    assert_eq!(value.array[1], 0);
}

#[derive(FromApp, State)]
struct Root;

#[derive(GlobUpdater)]
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
    fn apply(self, app: &mut App) {
        let glob = self.glob.get_mut(app);
        modor::update_field(&mut glob.integer, self.integer, &mut false);
        modor::update_field(&mut glob.dynamic, self.dynamic, &mut false);
        modor::update_field(&mut glob.array, self.array, &mut false);
    }
}
