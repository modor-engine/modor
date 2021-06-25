use microbench::Options;
use modor::*;

#[allow(clippy::cast_precision_loss)]
fn build_main_group(builder: &mut GroupBuilder<'_>) {
    for i in 0..10000 {
        if i % 10 == 0 {
            builder.with_entity::<DynamicBody>(i as f32);
        } else {
            builder.with_entity::<StaticBody>(i as f32);
        }
    }
}

struct StaticBody {
    pos_x: f32,
    pos_y: f32,
}

impl EntityMainComponent for StaticBody {
    type Data = f32;

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
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

    fn build(builder: &mut EntityBuilder<'_, Self>, data: Self::Data) -> Built {
        builder.inherit_from::<StaticBody>(data).with_self(Self {
            vel_x: data + 0.25,
            vel_y: data + 0.75,
        })
    }

    fn on_update(runner: &mut EntityRunner<'_, Self>) {
        runner.run(entity_system!(Self::update));
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
        Application::new().with_group(build_main_group)
    });

    let mut app = Application::new().with_group(build_main_group);
    microbench::bench(&options, "update", || app.update());
}
