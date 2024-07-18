use std::time::{Duration, Instant};

const OBJECT_COUNT: usize = 1_000_000;
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

fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    objects: Vec<Object>,
    last_frame_instant: Instant,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        app.get_mut::<Window>().set_title("Rendering 2D");
        Self {
            objects: (0..OBJECT_COUNT)
                .map(|index| Object::from_app_with(app, |object, app| object.init(app, index)))
                .collect(),
            last_frame_instant: Instant::now(),
        }
    }
}

impl Singleton for Root {
    fn update(&mut self, app: &mut App) {
        self.print_fps();
        for object in &mut self.objects {
            object.update(app);
        }
    }
}

impl Root {
    fn print_fps(&mut self) {
        let now = Instant::now();
        let fps = 1. / (now - self.last_frame_instant).as_secs_f32();
        info!("FPS: {}", fps);
        self.last_frame_instant = now;
    }
}

#[derive(FromApp)]
struct Object {
    model: Glob<Model2D>,
    next_update: Instant,
    // A `Body2D` could be used instead of manually handle the velocity, but for performance reasons
    // this is not recommended with a large amount of objects (> 10K objects).
    velocity: Vec2,
}

impl Object {
    fn init(&mut self, app: &mut App, index: usize) {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
        self.model
            .updater()
            .position(position)
            .size(Vec2::ONE * 0.01)
            .z_index(rng.gen_range(i16::MIN..i16::MAX))
            .material(app.get_mut::<Resources>().materials[index % COLORS.len()].to_ref())
            .apply(app);
        self.velocity = Vec2::ONE * 0.0001;
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
        self.model
            .updater()
            .for_position(app, |position| *position += self.velocity * delta)
            .apply(app);
    }
}

#[derive(Singleton)]
struct Resources {
    materials: Vec<GlobMaterial<DefaultMaterial2D>>,
}

impl FromApp for Resources {
    fn from_app(app: &mut App) -> Self {
        Self {
            materials: COLORS
                .iter()
                .map(|&color| Self::material(app, color))
                .collect(),
        }
    }
}

impl Resources {
    fn material(app: &mut App, color: Color) -> GlobMaterial<DefaultMaterial2D> {
        GlobMaterial::from_app(app, color)
            .with_update(|updater| updater.color(*color).is_ellipse(false).apply(app))
    }
}
