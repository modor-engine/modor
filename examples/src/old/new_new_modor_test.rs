#![allow(missing_docs)]

use instant::Instant;
use modor::log::Level;
use modor_examples::new_modor::{App, Data, Scope, Singleton};

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

// TODO: still not convinced for complex cases like Minecraft quadtree split in chunks
//    - current engine way better for this, try to start from the current engine ? (e.g replace globs by scope/index ?)
// TODO: can obtain another ScopedApp from &mut ScopedApp
// TODO: add post method when object accessed mutably
//    - is it possible with chaining ?
//    - get() -> return Option<&T>, don't run post method
//    - get_or_create() -> don't return Option<&T> but &T directly, don't run post method

fn update(app: &mut App) {
    let app = &mut app.scope(Scope::new("root"));
    let increment = app.single_mut::<Increment>().0;
    for index in 0..1_000_000 {
        app.get_mut::<Indexed>(index).0 += increment;
    }

    // let mut app = app.scope(Scope::new("root"));
    // let increment = app.single_mut::<Increment>().0;
    // for indexed in app.range_iter_mut::<Indexed>(0..1_000_000) {
    //     indexed.0 += increment;
    // }
}

struct Increment(u32);

impl Default for Increment {
    fn default() -> Self {
        Self(1)
    }
}

impl Singleton for Increment {}

#[derive(Default)]
struct Indexed(u32);

impl Data for Indexed {}
