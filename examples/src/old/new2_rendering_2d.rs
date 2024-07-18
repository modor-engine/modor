use std::time::{Duration, Instant};

// TODO: how to make it less verbose
//    - Try to put methods Singleton types instead of standalone methods

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
    modor_graphics::run(Level::Info, update);
}

fn update(app: &mut App) {
    app.run(print_fps);
    app.run(update_window);
    app.run(update_materials);
    app.run(update_objects);
}

fn print_fps(app: &mut App, state: &mut FpsState) {
    let now = Instant::now();
    let fps = 1. / (now - self.last_frame_instant).as_secs_f32();
    info!("FPS: {}", fps);
    state.last_frame_instant = now;
}

fn update_window(app: &mut App, window: &mut Window) {
    window.set_title("Rendering 2D");
}

fn update_materials(app: &mut App, materials: &mut Materials) {
    materials
        .0
        .scaled(app, COLORS.len())  // util trait provided by modor
        .iter()
        .enumerate()
        .for_each(|(index, material)| update_material(app, material, index));
}

fn update_material(app: &mut App, material: &Data<Mat<DefaultMaterial2D>>, index: usize) {
    Data::get_mut(app, material)
        .set_color(COLORS[index])
        .set_is_ellipse(true);
}

fn material(app: &mut App, index: usize) -> DataRef<Mat<DefaultMaterial2D>> {
    app.get_mut::<Materials>().0[index].clone()
}

fn update_objects(app: &mut App, objects: &mut Objects) {
    objects
        .0
        .scaled(app, OBJECT_COUNT)
        .iter()
        .enumerate()
        .for_each(|(index, object)| update_object(app, index, object, index));
}

fn update_object(app: &mut App, object: &mut Object, index: usize) {
    object.model.on_init(|| init_object(app, object, index)); // TODO: avoid having update state in Data<T>
    let offset = velocity(object) * app.get_mut::<Delta>().duration().as_secs_f32();
    Data::get_mut(app, &object.model).update_position(|position| position + offset);
}

fn init_object(app: &mut App, object: &mut Object, index: usize) {
    let mut rng = rand::thread_rng();
    let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
    let material = material(app, index % COLORS.len());
    Data::get_mut(app, &object.model)
        .set_position(position)
        .set_size(Vec2::ONE * 0.01)
        .set_z_index(rng.gen_range(i16::MIN..i16::MAX))
        .set_material(material);
    // TODO: how to have methods like on_click(||) ?
}

fn velocity(object: &mut Object) -> Vec2 {
    if Instant::now() > object.next_update {
        let mut rng = rand::thread_rng();
        object.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
            .with_magnitude(0.05)
            .unwrap_or(Vec2::ZERO);
        object.next_update = Instant::now() + Duration::from_millis(rng.gen_range(200..400));
    }
    object.velocity
}

#[derive(Default, Singleton)] // impl<T: Default> FromApp for Default {}
struct FpsState {
    last_frame_instant: Instant,
}

#[derive(AppDefault, Singleton)]
struct Materials(Vec<Data<Mat<DefaultMateral2D>>>);

#[derive(AppDefault, Singleton)]
struct Objects(Vec<Object>);

#[derive(AppDefault)]
struct Object {
    model: Data<Model2D>,
    next_update: Instant,
    // A `Body2D` could be used instead of manually handle the velocity, but for performance reasons
    // this is not recommended with a large amount of objects (> 10K objects).
    velocity: Vec2,
}
