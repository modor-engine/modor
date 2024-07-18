#![allow(missing_docs)]

// TODO: remove this example

use modor::log::Level;
use modor::{App, FromApp, Glob, Singleton, Updater};
use std::marker::PhantomData;
use std::ops::AddAssign;

fn main() {
    let mut app = App::new(Level::Info);
    app.create::<Root>();
    app.update();
    app.update();
}

#[derive(FromApp)]
struct Root {
    value: usize,
    data: Vec<Glob<Data<u32>>>,
}

impl Singleton for Root {
    fn init(&mut self, app: &mut App) {
        self.value = 42;
        for value in 0..10 {
            self.data
                .push(Glob::<Data<u32>>::from_app_with(app, |data, app| {
                    Self::init_data(data, app, value);
                }));
        }
    }

    fn update(&mut self, app: &mut App) {
        self.value += app.get_mut::<Increment>().0;
        println!("Value: {}", self.value);
        for data in &self.data {
            println!("Data: {:?}", data.get(app));
        }
    }
}

impl Root {
    fn init_data(data: &Glob<Data<u32>>, app: &mut App, value: u32) {
        data.updater()
            .integer(value)
            .additional_integer(10_u32)
            .for_string(app, |string| *string = value.to_string())
            .apply(app);
    }
}

#[derive(Singleton)]
struct Increment(usize);

impl FromApp for Increment {
    fn from_app(_app: &mut App) -> Self {
        Self(1)
    }
}

#[derive(Debug, FromApp, Updater)]
pub struct Data<T: 'static> {
    /// Internal value stored in a data object.
    #[updater(field, for_field = "default")]
    integer: u32,
    /// Internal label stored in a data object.
    #[updater(field, for_field = "default")]
    string: String,
    /// Virtual field for the updater.
    #[updater(inner_type, field, for_field = "|_, _| todo!()")]
    additional_integer: PhantomData<T>,
}

impl<T: 'static> DataUpdater<'_, T>
where
    u32: AddAssign<T>,
{
    fn apply(mut self, app: &mut App) {
        let data = self.glob.get_mut(app);
        let mut is_updated = false;
        modor::update_field(&mut data.integer, self.integer.take(), &mut is_updated);
        modor::update_field(&mut data.string, self.string.take(), &mut is_updated);
        if let Some(generics) = self.additional_integer {
            data.integer += generics;
        }
        if is_updated {
            println!("Data {} has been updated", self.glob.index());
        }
    }
}
