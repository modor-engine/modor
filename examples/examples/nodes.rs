#![allow(missing_docs)]

use modor::log::{info, Level};
use modor::{App, Context, Node, RootNode, Visit};
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let mut app = App::new::<Root>(Level::Info);
    info!("Build time: {:?}", start.elapsed());
    let start = Instant::now();
    let update_count = 100;
    for _ in 0..update_count {
        app.update();
    }
    info!("Update time: {:?}", start.elapsed() / update_count);
}

#[derive(Node, Visit)]
struct Root {
    numbers: Vec<Number>,
}

impl RootNode for Root {
    fn on_create(_ctx: &mut Context<'_>) -> Self {
        Self {
            numbers: (0..1_000_000).map(|_| Number::default()).collect(),
        }
    }
}

#[derive(Default, Visit)]
struct Number {
    #[modor(skip)]
    value: u128,
    inner: InnerValue,
}

impl Node for Number {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.value += 1;
    }
}

#[derive(Default, Visit)]
struct InnerValue {
    #[modor(skip)]
    value: u128,
}

impl Node for InnerValue {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.value += 1;
    }
}
