use std::any::TypeId;
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
const SCOPE: Scope = Scope::new("root");

fn main() {
    modor_graphics::run(Level::Info, root);
}

fn root(app: &mut App) {
    app.run(fps_display);
    materials();
    objects();
}

fn fps_display(app: &mut App, state: &mut State) {
    let now = Instant::now();
    let fps = 1. / (now - state.last_frame_instant).as_secs_f32();
    state.last_frame_instant = now;
    info!("FPS: {}", fps);
}

fn materials(app: &mut App) {
    for (index, color) in COLOR.iter().enumerate() {
        Mat::<DefaultMaterial2D>::desc(SCOPE.key(index))
            .color(color)
            .is_ellipse(true);
    }
}

fn objects(app: &mut App) {
    for index in 0..OBJECT_COUNT {
        object(app, index);
    }
}

// More optimized version
fn objects(app: &mut App) {
    let delta = app.get_mut::<Delta>().duration().as_secs_f32();
    Model2D::create(0..OBJECT_COUNT);
    ObjectVelocity::create(0..OBJECT_COUNT);
    // TODO: issue: Storage<Model2D> does not borrow InstanceGroups, how to do it ?
    app.run::<Storage<Model2D>>(|app, models| {
        for (model, velocity) in models
            .scoped_iter_mut(SCOPE)
            .zip(ObjectVelocity::scoped_iter_mut(SCOPE))
        {
            object(model, velocity, delta);
        }
    });
}

fn object(app: &mut App, index: usize) {
    let delta = app.get_mut::<Delta>().duration().as_secs_f32();
    let velocity = ObjectVelocity::desc(SCOPE.key(index)).get();
    Model2D::desc(SCOPE.key(index))
        .on_init(initial_object_model)
        .for_position(|position| *position = position + velocity * delta);
}

fn initial_object_model(model: &mut Model2D) {
    let mut rng = rand::thread_rng();
    let position = Vec2::new(rng.gen_range(-0.2..0.2), rng.gen_range(-0.2..0.2));
    model
        .position(position)
        .size(Vec2::ONE * 0.01)
        .z_index(rng.gen_range(i16::MIN..i16::MAX))
        .material(SCOPE.key(index % COLORS.len()));
}

#[derive(AppDefault, Singleton)] // possibility to add callback with app.on_update(my_fn); in AppDefault::app_default()
struct State {
    last_frame_instant: Instant,
}

#[derive(VecData)]
struct ObjectVelocity {
    next_update: Instant,
    velocity: Vec2,
}

impl Default for ObjectVelocity {
    fn default() -> Self {
        Self {
            next_update: Instant::now(),
            velocity: Vec2::ZERO,
        }
    }
}

impl ObjectVelocity {
    fn get(&mut self) -> Vec2 {
        let now = Instant::now();
        if now > self.next_update {
            let mut rng = rand::thread_rng();
            self.velocity = Vec2::new(rng.gen_range(-0.5..0.5), rng.gen_range(-0.5..0.5))
                .with_magnitude(0.05)
                .unwrap_or(Vec2::ZERO);
            self.next_update = now + Duration::from_millis(rng.gen_range(200..400));
        }
        self.velocity
    }
}

// Singleton storage: FxHashMap<TypeId, FxHashMap<Scope, T>>
//    Can be more optimized -> FxHashMap<TypeScope, T> (hash: 32bit TypeId + 32bit Scope)
//      + track list of TypeScope for each scope (for deletion)
//      + track list of TypeScope for each type (for iteration)
// The scope type can be configured with Singleton trait
//    - Scope can be removed (implement DeletableScope) -> e.g. for Storage<T>
//    - () cannot be removed (does not implement DeletableScope) -> e.g. for CollisionGroups

// For testing, as the object doesn't have any getter, the attributes can be tested using the following macro:
// macro_rules! modor_assert_eq {
//     ($object:expr, $method:ident, $expected:expr) => {{
//         let mut called = false;
//         $object.$method(|value| {
//             assert_eq!(*value, $expected);
//             called = true;
//         });
//         assert!(called, concat!("callback for ", stringify!($method), " was not called"));
//     }};
// }

// Collision groups uses group keys to identify each group.
// A group key is composed of a "scope" (not the same type as Scope) and an index.

// how to borrow multiple data types ? (e.g. Text2D borrow Model2D and Texture and Mat)
//     -> Create data redundancy, and update necessary nested objects when object reference is dropped
//     -> So e.g., need to wrap Model2D in Model2DDesc
//     -> In for_X, need to force borrow to access the existing value
