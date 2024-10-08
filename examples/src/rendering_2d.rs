use instant::Instant;
use modor::log::{info, Level};
use modor::{App, FromApp, State};
use modor_graphics::{
    Color, DefaultMaterial2D, DefaultMaterial2DUpdater, MatGlob, Model2D, Window,
};
use modor_physics::modor_math::Vec2;
use modor_physics::Delta;
use rand::Rng;
use std::time::Duration;

const SPRITE_COUNT: usize = 1_000;
const COLORS: [Color; 10] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::WHITE,
    Color::YELLOW,
    Color::CYAN,
    Color::PURPLE,
    Color::MAROON,
    Color::GRAY,
    Color::OLIVE,
];

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    objects: Vec<Object>,
    last_frame_instant: Instant,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        app.get_mut::<Window>().title = "Rendering 2D".into();
        Self {
            objects: (0..SPRITE_COUNT)
                .map(|index| Object::new(app, index))
                .collect(),
            last_frame_instant: Instant::now(),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        let now = Instant::now();
        info!(
            "FPS: {}",
            1. / (now - self.last_frame_instant).as_secs_f32()
        );
        self.last_frame_instant = now;
        for object in &mut self.objects {
            object.update(app);
        }
    }
}

#[derive(FromApp)]
struct Resources {
    materials: Vec<MatGlob<DefaultMaterial2D>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        self.materials = COLORS
            .iter()
            .map(|&color| {
                MatGlob::from_app_with(app, |mat, app| {
                    DefaultMaterial2DUpdater::default()
                        .color(color)
                        .is_ellipse(true)
                        .apply(app, mat);
                })
            })
            .collect();
    }
}

struct Object {
    model: Model2D,
    next_update: Instant,
    // A `Body2D` could be used instead of manually handle the velocity, but for performance reasons
    // this is not recommended with a large amount of objects (> 10K objects).
    velocity: Vec2,
}

impl Object {
    fn new(app: &mut App, index: usize) -> Self {
        let mut rng = rand::thread_rng();
        let material = app.get_mut::<Resources>().materials[index % COLORS.len()].to_ref();
        let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
        let model = Model2D::new(app)
            .with_material(material.to_ref())
            .with_position(position)
            .with_size(Vec2::ONE * 0.01)
            .with_z_index(rng.gen_range(i16::MIN..i16::MAX));
        Self {
            model,
            next_update: Instant::now(),
            velocity: Vec2::ONE * 0.0001,
        }
    }

    fn update(&mut self, app: &mut App) {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            self.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
            self.next_update = Instant::now() + Duration::from_millis(rng.gen_range(200..400));
        }
        let delta = app.get_mut::<Delta>().duration.as_secs_f32();
        self.model.position += self.velocity * delta;
        self.model.update(app);
    }
}
