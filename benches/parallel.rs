//! Benchmark parallel execution of systems

use microbench::Options;
use modor::*;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct Reader(f32);

#[derive(Debug)]
struct Writer1(f32);

#[derive(Debug)]
struct Writer2(f32);

struct MainEntity;

impl EntityMainComponent for MainEntity {
    type Data = ();

    fn build(builder: EntityBuilder<'_, Self>, _: Self::Data) -> Built {
        builder
            .with(Reader(42.))
            .with(Writer1(0.))
            .with(Writer2(0.))
            .with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::update1));
        runner.run(system!(Self::update2));
    }
}

impl MainEntity {
    fn update1(writer: &mut Writer1, reader: &Reader) {
        writer.0 = reader.0;
        thread::sleep(Duration::from_nanos(1));
    }

    fn update2(writer: &mut Writer2, reader: &Reader) {
        writer.0 = reader.0;
        thread::sleep(Duration::from_nanos(1));
    }
}

fn main() {
    let options = Options::default();

    let mut app = App::new().with_thread_count(1);
    for _ in 0..1000 {
        app = app.with_entity::<MainEntity>(());
    }
    microbench::bench(&options, "sequential update", || app.update());

    let mut app = App::new().with_thread_count(2);
    for _ in 0..1000 {
        app = app.with_entity::<MainEntity>(());
    }
    microbench::bench(&options, "parallel update", || app.update());
}
