#![allow(missing_docs)]

use instant::Instant;
use modor::log::Level;
use modor_examples::new_modor::{App, Data, Scope, SingletonStorage, VecStorage};

const SCOPE: Scope = Scope::new("root");

fn main() {
    let start = Instant::now();
    for _ in 0..100 {
        App::new(Level::Info, update).update();
    }
    println!("Build time: {:?}", start.elapsed() / 100);
    let start = Instant::now();
    let mut app = App::new(Level::Info, update);
    app.update();
    for _ in 0..100 {
        app.update();
    }
    println!("Update time: {:?}", start.elapsed() / 100);
}

fn update(app: &mut App) {
    for index in 0..1_000_000 {
        // let increment = Increment::get(app);
        let increment = Increment::get_mut(app, ()).0;
        Indexed::get_mut(app, SCOPE.key(index)).0 += increment;
    }

    // let increment = Increment::get_mut(app, ()).0;
    // Indexed::scale(app, SCOPE.key(1_000_000));
    // for indexed in Indexed::iter_mut(app) {
    //     indexed.0 += increment;
    // }

    // Indexed::scale(app, SCOPE.key(1_000_000));
    // Indexed::take_scope_each(app, SCOPE, |app, indexed| {
    //     Increment::take(app, (), |app, increment| {
    //         indexed.0 += increment.0;
    //     });
    // });

    // Indexed::scale(app, SCOPE.key(1_000_000));
    // Increment::take(app, (), |app, increment| {
    //     Indexed::take_scope_each(app, SCOPE, |_, indexed| {
    //         indexed.0 += increment.0;
    //     });
    // });
}

struct Increment(u32);

impl Default for Increment {
    fn default() -> Self {
        Self(1)
    }
}

// impl Increment {
//     fn get(app: &mut App) -> u32 {
//         <Self as Data>::get_mut(app, Scope::new("").key(0)).0
//     }
// }

impl Data for Increment {
    type Storage = SingletonStorage<Self>;
}

#[derive(Default)]
struct Indexed(u32);

impl Data for Indexed {
    type Storage = VecStorage<Self>;
}
