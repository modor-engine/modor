use std::time::{Duration, Instant};

// impl<T: Default> FromApp for Default {}

// TODO: conclusion on this example
//     - Not really better than current engine
//     - Having singletons everywhere is still verbose, compare to new_rendering_2d for example
//     - We need to keep new_rendering_2d internals simple if this should be put in place, but also flexible
//         - E.g. for Minecraft
//         - for perfs, app can be scoped (need to see how to make it not very verbose in new_rendering_2d)
//         - Maybe we can type Key<T>, and allow automatic creation of keys for complex cases
//         - app.get_mut::<T>() shouldn't do anything magic, only track mutated object for later iteration
//             - ISSUE: how to make sure we don't miss some mutations ? (be outside mutation<->reset time range)
//         - Have most methods taking only app: &mut App param is preferred most of the time

const OBJECT_COUNT: usize = 1_000;
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

#[derive(AppDefault)]
struct Root;

impl Singleton for Root {
    fn init(&mut self, app: &mut App) {
        app.create::<Fps>();
        app.create::<Materials>();
        app.create::<Objects>();
        app.get_mut::<Window>().set_title("Rendering 2D");
    }
}

#[derive(AppDefault)]
struct Fps {
    last_frame_instant: Instant,
}

impl Default for Fps {
    fn default() -> Self {
        Self {
            last_frame_instant: Instant::now(),
        }
    }
}

impl Singleton for Fps {
    fn update(&mut self, app: &mut App) {
        let now = Instant::now();
        let fps = 1. / (now - self.last_frame_instant).as_secs_f32();
        info!("FPS: {}", fps);
        self.last_frame_instant = now;
    }
}

#[derive(AppDefault)]
struct Materials(Vec<Data<Mat<DefaultMaterial2D>>>);

impl Singleton for Materials {
    fn init(&mut self, app: &mut App) {
        self.0.resize_with(COLORS.len(), || Data::new());
        for (index, material) in self.0.iter_mut().enumerate() {
            Data::get_mut(app, material)
                .set_color(COLORS[index])
                .set_is_ellipse(true);
        }
    }
}

#[derive(AppDefault)]
struct Objects(Vec<Object>);

impl Singleton for Objects {
    fn init(&mut self, app: &mut App) {
        self.0
            .resize_with(OBJECT_COUNT, || Object::app_default(app));
        for (index, object) in self.0.iter_mut().enumerate() {
            object.init(app);
        }
    }
}

#[derive(AppDefault)]
struct Object {
    model: Data<Model2D>,
    next_update: Instant,
    // A `Body2D` could be used instead of manually handle the velocity, but for performance reasons
    // this is not recommended with a large amount of objects (> 10K objects).
    velocity: Vec2,
}

impl Object {
    fn init(&mut self, app: &mut App) -> Self {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
        let material = material(app, index % COLORS.len());
        Data::get_mut(app, &self.model)
            .set_position(position)
            .set_size(Vec2::ONE * 0.01)
            .set_z_index(rng.gen_range(i16::MIN..i16::MAX))
            .set_material(material);
    }

    fn update(&mut self, app: &mut App) -> Self {
        if Instant::now() > self.next_update {
            let mut rng = rand::thread_rng();
            self.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
            self.next_update = Instant::now() + Duration::from_millis(rng.gen_range(200..400));
        }
        let offset = self.velocity * app.get_mut::<Delta>().duration().as_secs_f32();
        Data::get_mut(app, &self.model).update_position(|position| position + offset);
    }
}
