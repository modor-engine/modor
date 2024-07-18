use std::time::Duration;

// TODO: SCOPE is not a very flexible way in complex cases like Minecraft.

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
const SCOPE: Scope = Scope::new("root");

fn main() {
    modor_graphics::run(Level::Info, update);
}

fn update(app: &mut App) {
    print_fps();
    update_materials();
    update_objects();
}

fn print_fps(app: &mut App) {
    let now = Instant::now();
    let state = app.get_mut::<State>(());
    let fps = 1. / (now - state.last_frame_instant).as_secs_f32();
    info!("FPS: {}", fps);
    state.last_frame_instant = now;
}

fn update_materials(app: &mut App) {
    for (index, color) in COLOR.iter().enumerate() {
        app.get_mut::<Mat<DefaultMaterial2D>>(SCOPE.key(index))
            .set_color(color)
            .set_is_ellipse(true)
    }
}

fn update_objects(app: &mut App) {
    for index in 0..OBJECT_COUNT {
        update_object(app, index);
    }
}

fn update_object(app: &mut App, index: usize) {
    init_object(app, index);
    let delta = app.get_mut::<Delta>().duration().as_secs_f32();
    let velocity = object_velocity(app, index);
    let mut position = Vec2::ZERO;
    app.get_mut::<Model2D>(index)
        .get_position(&mut position)
        .set_position(position + velocity * delta);
}

fn init_object(app: &mut App, index: usize) {
    if let Some(model) = app.init::<Model2D>(index) {
        let mut rng = rand::thread_rng();
        let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
        model
            .set_position(position)
            .set_size(Vec2::ONE * 0.01)
            .set_z_index(rng.gen_range(i16::MIN..i16::MAX))
            .set_material(SCOPE.key(index % COLORS.len()));
    }
}

fn object_velocity(app: &mut App, index: usize) {
    let object = app.get_mut::<Object>(index);
    if Instant::now() > object.next_update {
        let mut rng = rand::thread_rng();
        object.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
            .with_magnitude(0.05)
            .unwrap_or(Vec2::ZERO);
        object.next_update = Instant::now() + Duration::from_millis(rng.gen_range(200..400));
    }
    object.velocity
}

#[derive(Default, SingletonData)]
struct State {
    last_frame_instant: Instant,
}

#[derive(Default, VecData)]
struct Object {
    next_update: Instant,
    velocity: Vec2,
}
