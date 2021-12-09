//! Benchmark system iteration

use microbench::Options;
use modor::*;

struct StaticBody {
    pos_x: f32,
    pos_y: f32,
}

impl EntityMainComponent for StaticBody {
    type Data = f32;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.with_self(Self {
            pos_x: data,
            pos_y: data + 0.5,
        })
    }
}

struct DynamicBody {
    vel_x: f32,
    vel_y: f32,
}

impl EntityMainComponent for DynamicBody {
    type Data = f32;

    fn build(builder: EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.inherit_from::<StaticBody>(data).with_self(Self {
            vel_x: data + 0.25,
            vel_y: data + 0.75,
        })
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(system!(Self::update));
    }
}

impl DynamicBody {
    fn update(&self, static_body: &mut StaticBody) {
        static_body.pos_x += self.vel_x;
        static_body.pos_y += self.vel_y;
    }
}

fn main() {
    let options = Options::default();

    microbench::bench(&options, "build", || {
        let mut app = App::new();
        for i in 0..10000_u16 {
            if i % 10 == 0 {
                app = app.with_entity::<DynamicBody>(f32::from(i));
            } else {
                app = app.with_entity::<StaticBody>(f32::from(i));
            }
        }
        app
    });

    let mut app = App::new();
    for i in 0..10000_u16 {
        if i % 10 == 0 {
            app = app.with_entity::<DynamicBody>(f32::from(i));
        } else {
            app = app.with_entity::<StaticBody>(f32::from(i));
        }
    }
    microbench::bench(&options, "update", || app.update());
}
