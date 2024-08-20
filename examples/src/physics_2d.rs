use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_input::{Inputs, MouseButton};
use modor_graphics::{Color, CursorTracker, DefaultMaterial2DUpdater, Sprite2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::{
    Body2D, Body2DUpdater, CollisionGroup, CollisionGroupUpdater, Impulse, Shape2D,
};
use rand::Rng;

const GRAVITY: f32 = 2.;
const CANNON_JOIN_POSITION: Vec2 = Vec2::new(0., 0.6);
const CANNON_LENGTH: f32 = 0.3;
const OBJECT_MASS: f32 = 10.;
const OBJECT_RADIUS: f32 = 0.04;
const OBJECT_INITIAL_SPEED: f32 = 1.;

const RECTANGLE_INERTIA_FACTOR: f32 = 1. / 3.;
const CIRCLE_INERTIA_FACTOR: f32 = 1. / 4.;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root {
    left_wall: Wall,
    right_wall: Wall,
    bottom_wall: Wall,
    cannon: Cannon,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.left_wall
            .init(app, Vec2::X * -0.5, Vec2::new(0.03, 1.));
        self.right_wall
            .init(app, Vec2::X * 0.5, Vec2::new(0.03, 1.));
        self.bottom_wall
            .init(app, Vec2::Y * -0.5, Vec2::new(1., 0.03));
        self.cannon.init();
        Self::init_anti_aliasing(app);
    }

    fn update(&mut self, app: &mut App) {
        self.left_wall.update(app);
        self.right_wall.update(app);
        self.bottom_wall.update(app);
        self.cannon.update(app);
    }
}

impl Root {
    fn init_anti_aliasing(app: &mut App) {
        app.take::<Window, _>(|window, app| {
            let target = window.target.get_mut(app);
            target.anti_aliasing = target
                .supported_anti_aliasing_modes()
                .iter()
                .copied()
                .max()
                .unwrap_or_default();
        });
    }
}

#[derive(FromApp)]
struct CollisionGroups {
    wall: Glob<CollisionGroup>,
    object: Glob<CollisionGroup>,
}

impl State for CollisionGroups {
    fn init(&mut self, app: &mut App) {
        let impulse = Impulse::new(0.1, 0.8);
        CollisionGroupUpdater::new(&self.object)
            .add_impulse(app, &self.wall, impulse)
            .add_impulse(app, &self.object, impulse);
    }
}

#[derive(FromApp)]
struct Wall {
    body: Glob<Body2D>,
    sprite: Sprite2D,
}

impl Wall {
    fn init(&mut self, app: &mut App, position: Vec2, size: Vec2) {
        Body2DUpdater::default()
            .position(position)
            .size(size)
            .collision_group(app.get_mut::<CollisionGroups>().wall.to_ref())
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}

struct Cannon {
    sprite: Sprite2D,
    cursor: CursorTracker,
}

impl FromApp for Cannon {
    fn from_app(app: &mut App) -> Self {
        Self {
            sprite: Sprite2D::from_app(app),
            cursor: CursorTracker::new(app),
        }
    }
}

impl Cannon {
    fn init(&mut self) {
        self.sprite.model.size = Vec2::new(0.05, CANNON_LENGTH);
    }

    fn update(&mut self, app: &mut App) {
        let cursor_position = self.cursor.position(app);
        self.sprite.model.rotation = Vec2::Y.rotation(cursor_position - CANNON_JOIN_POSITION);
        self.sprite.model.position = CANNON_JOIN_POSITION
            + (Vec2::Y * CANNON_LENGTH / 2.).with_rotation(self.sprite.model.rotation);
        self.create_object(app, self.sprite.model.rotation);
        self.sprite.update(app);
        self.cursor.update(app);
    }

    fn create_object(&self, app: &mut App, rotation: f32) {
        let position = CANNON_JOIN_POSITION
            + (Vec2::Y * (CANNON_LENGTH + OBJECT_RADIUS / 2.)).with_rotation(rotation);
        let velocity = Vec2::Y.with_rotation(rotation) * OBJECT_INITIAL_SPEED;
        let object = if self.cursor.state(app).is_just_released() {
            Some(Object::from_app_with(app, |object, app| {
                object.init(app, position, velocity, false);
            }))
        } else if app.get_mut::<Inputs>().mouse[MouseButton::Right].is_just_released() {
            Some(Object::from_app_with(app, |object, app| {
                object.init(app, position, velocity, true);
            }))
        } else {
            None
        };
        app.get_mut::<Objects>().objects.extend(object);
    }
}

#[derive(FromApp)]
struct Objects {
    objects: Vec<Object>,
}

impl State for Objects {
    fn update(&mut self, app: &mut App) {
        self.objects.retain_mut(|object| {
            object.update(app);
            object.body.get(app).position(app).y > -5.
        });
    }
}

#[derive(FromApp)]
struct Object {
    body: Glob<Body2D>,
    sprite: Sprite2D,
}

impl Object {
    fn init(&mut self, app: &mut App, position: Vec2, velocity: Vec2, is_ball: bool) {
        let mut rng = rand::thread_rng();
        let (inertia_factor, shape) = if is_ball {
            (CIRCLE_INERTIA_FACTOR, Shape2D::Circle)
        } else {
            (RECTANGLE_INERTIA_FACTOR, Shape2D::Rectangle)
        };
        Body2DUpdater::default()
            .position(position)
            .size(Vec2::ONE * OBJECT_RADIUS * 2.)
            .velocity(velocity)
            .force(-Vec2::Y * GRAVITY * OBJECT_MASS)
            .mass(OBJECT_MASS)
            .angular_inertia(OBJECT_MASS * OBJECT_RADIUS.powi(2) / inertia_factor)
            .collision_group(app.get_mut::<CollisionGroups>().object.to_ref())
            .shape(shape)
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        DefaultMaterial2DUpdater::default()
            .is_ellipse(is_ball)
            .color(Color::rgb(
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
            ))
            .apply(app, &self.sprite.material);
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
    }
}
