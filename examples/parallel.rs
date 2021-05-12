use modor::*;

fn main() {
    Application::default()
        .with_group(build_main_group)
        .with_thread_count(2)
        .update();
}

fn build_main_group(builder: &mut GroupBuilder<'_>) {
    for _ in 0..10 {
        builder.with_entity::<MainEntity>(());
    }
}

#[derive(Debug)]
struct Reader(f32);

#[derive(Debug)]
struct Writer1(f32);

#[derive(Debug)]
struct Writer2(f32);

struct MainEntity;

impl EntityMainComponent for MainEntity {
    type Params = ();

    fn build(builder: &mut EntityBuilder<'_, Self>, _: Self::Params) -> Built {
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
        println!("Update1");
    }

    fn update2(writer: &mut Writer2, reader: &Reader) {
        writer.0 = reader.0;
        println!("Update2");
    }
}
