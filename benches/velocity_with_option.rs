use microbench::Options;
use modor::*;

fn main_group_builder() -> impl FnOnce(&mut GroupBuilder<'_>) {
    |builder| {
        for i in 0..10000 {
            if i % 10 == 0 {
                builder.with_entity::<DynamicBody>(i);
            } else {
                builder.with_entity::<StaticBody>(i);
            }
        }
    }
}

#[derive(Debug)]
struct Position(f32, f32);

#[derive(Debug)]
struct Velocity(f32, f32);

struct StaticBody;

impl LightEntity for StaticBody {
    type LightParams = usize;

    fn build(builder: &mut LightEntityBuilder<'_, Self>, value: Self::LightParams) {
        builder.with(Position(value as f32, value as f32 + 0.5));
    }
}

struct DynamicBody;

impl Entity for DynamicBody {
    type Params = usize;

    fn build(builder: &mut EntityBuilder<'_, Self>, value: Self::Params) -> Built {
        builder
            .inherit_from::<StaticBody>(value)
            .with(Velocity(value as f32 + 0.25, value as f32 + 0.75))
            .with_self(Self)
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::update));
    }
}

impl DynamicBody {
    fn update(position: Option<&mut Position>, velocity: &Velocity) {
        if let Some(position) = position {
            position.0 += velocity.0;
            position.1 += velocity.1;
        }
    }
}

fn main() {
    let options = Options::default();

    let mut app = Application::default().with_group(main_group_builder());
    microbench::bench(&options, "update with option", || app.update());
}
