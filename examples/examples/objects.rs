#![allow(missing_docs)]

// TODO: remove

use instant::Instant;
use modor::log::{info, LevelFilter};
use modor::{App, Context, Id, NoRole, Object, SingletonObject};

#[modor::main]
fn main() {
    let mut app = App::new();
    app.set_log_level(LevelFilter::Trace);
    let start = Instant::now();
    app.create(Main::new);
    info!("Creation time: {:?}", start.elapsed());
    let start = Instant::now();
    for _ in 0..100 {
        app.update();
    }
    info!("Average update time: {:?}", start.elapsed() / 100);
}

#[derive(SingletonObject)]
struct Main;

impl Main {
    fn new(ctx: &mut Context<'_, Self>) -> Self {
        for _ in 0..1_000_000 {
            ctx.create(Parent::new);
        }
        Self
    }
}

struct Parent {
    child: Id<Child>,
    value: u32,
}

impl Object for Parent {
    type Role = NoRole;

    fn update(&mut self, ctx: &mut Context<'_, Self>) -> modor::Result<()> {
        self.value += self.child.get(ctx)?.child.get(ctx)?.value;
        Ok(())
    }
}

impl Parent {
    fn new(ctx: &mut Context<'_, Self>) -> Self {
        Self {
            child: ctx.create(Child::new),
            value: 0,
        }
    }
}

#[derive(Object)]
struct Child {
    child: Id<GrandChild>,
}

impl Child {
    fn new(ctx: &mut Context<'_, Self>) -> Self {
        Self {
            child: ctx.create(|_| GrandChild::new()),
        }
    }
}

#[derive(Object)]
struct GrandChild {
    value: u32,
}

impl GrandChild {
    fn new() -> Self {
        Self { value: 5 }
    }
}
