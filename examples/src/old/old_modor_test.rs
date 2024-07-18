#![allow(missing_docs)]

use instant::Instant;
use modor::log::Level;
use modor::{App, Context, Node, RootNode, Visit};

fn main() {
    let start = Instant::now();
    for _ in 0..100 {
        App::new::<Root>(Level::Info).update();
    }
    println!("Build time: {:?}", start.elapsed() / 100);
    let start = Instant::now();
    let mut app = App::new::<Root>(Level::Info);
    app.update();
    for _ in 0..100 {
        app.update();
    }
    println!("Update time: {:?}", start.elapsed() / 100);
}

#[derive(Visit)]
struct Root {
    indexed: Vec<u32>,
}

impl RootNode for Root {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self {
            indexed: vec![0; 1_000_000],
        }
    }
}

impl Node for Root {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        for value in &mut self.indexed {
            let increment = ctx.get_mut::<Increment>().0;
            *value += increment;
        }
    }
}

#[derive(RootNode, Node, Visit)]
struct Increment(u32);

impl Default for Increment {
    fn default() -> Self {
        Self(1)
    }
}
