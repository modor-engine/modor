use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::{Color, CursorTracker, Sprite2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::{
    Body2D, Body2DUpdater, Collision2D, CollisionGroup, CollisionGroupUpdater, Shape2D,
};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root {
    rectangle: Shape,
    circle: Shape,
    cursor: Cursor,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.rectangle
            .init(app, Vec2::X * 0.25, Vec2::new(0.2, 0.3), false);
        self.circle
            .init(app, -Vec2::X * 0.25, Vec2::ONE * 0.4, true);
        self.cursor.init(app);
        app.get_mut::<Window>().is_cursor_visible = false;
    }

    fn update(&mut self, app: &mut App) {
        self.rectangle.update(app);
        self.circle.update(app);
        self.cursor.update(app);
    }
}

#[derive(FromApp)]
struct CollisionGroups {
    shape: Glob<CollisionGroup>,
    cursor: Glob<CollisionGroup>,
}

impl State for CollisionGroups {
    fn init(&mut self, app: &mut App) {
        CollisionGroupUpdater::new(&self.cursor).add_sensor(app, &self.shape);
    }
}

struct Shape {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    collisions: Vec<CollisionNormal>,
}

impl FromApp for Shape {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            collisions: vec![],
        }
    }
}

impl Shape {
    fn init(&mut self, app: &mut App, position: Vec2, size: Vec2, is_circle: bool) {
        Body2DUpdater::default()
            .position(position)
            .size(size)
            .collision_group(app.get_mut::<CollisionGroups>().shape.to_ref())
            .shape(if is_circle {
                Shape2D::Circle
            } else {
                Shape2D::Rectangle
            })
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        self.sprite.material.is_ellipse = is_circle;
        self.sprite.material.color = Color::CYAN;
    }

    fn update(&mut self, app: &mut App) {
        self.sprite.update(app);
        self.collisions.clear();
        self.body.take(app, |body, app| {
            for collision in body.collisions() {
                let collision = CollisionNormal::new(app, collision, false);
                self.collisions.push(collision);
            }
        });
    }
}

struct Cursor {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    collisions: Vec<CollisionNormal>,
    tracker: CursorTracker,
}

impl FromApp for Cursor {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            collisions: vec![],
            tracker: CursorTracker::new(app),
        }
    }
}

impl Cursor {
    fn init(&mut self, app: &mut App) {
        Body2DUpdater::default()
            .size(Vec2::new(0.05, 0.1))
            .rotation(FRAC_PI_8)
            .collision_group(app.get_mut::<CollisionGroups>().cursor.to_ref())
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        self.sprite.model.z_index = 1;
        self.sprite.material.color = Color::GREEN;
    }

    fn update(&mut self, app: &mut App) {
        self.tracker.update(app);
        Body2DUpdater::default()
            .position(self.tracker.position(app))
            .apply(app, &self.body);
        self.body.take(app, |body, app| {
            self.sprite.material.color = if body.collisions().is_empty() {
                Color::GREEN
            } else {
                Color::RED
            };
            self.collisions.clear();
            for collision in body.collisions() {
                self.collisions
                    .push(CollisionNormal::new(app, collision, true));
            }
        });
        self.sprite.update(app);
    }
}

struct CollisionNormal {
    _position: Sprite2D,
    _penetration: Sprite2D,
}

impl CollisionNormal {
    fn new(app: &mut App, collision: &Collision2D, from_cursor: bool) -> Self {
        let z_index = if from_cursor { 2 } else { 3 };
        let color = if from_cursor {
            Color::YELLOW
        } else {
            Color::DARK_GRAY
        };
        let lateral_offset = collision
            .penetration
            .with_rotation(FRAC_PI_2)
            .with_magnitude(0.0025)
            .unwrap_or_default();
        let penetration_position = collision.position - collision.penetration / 2. + lateral_offset;
        let mut position = Sprite2D::new(app)
            .with_model(|m| m.position = collision.position)
            .with_model(|m| m.size = Vec2::ONE * 0.02)
            .with_model(|m| m.z_index = z_index)
            .with_material(|m| m.color = color)
            .with_material(|m| m.is_ellipse = true);
        position.update(app);
        let mut penetration = Sprite2D::new(app)
            .with_model(|m| m.position = penetration_position)
            .with_model(|m| m.size = Vec2::new(0.005, collision.penetration.magnitude()))
            .with_model(|m| m.rotation = Vec2::Y.rotation(-collision.penetration))
            .with_model(|m| m.z_index = z_index)
            .with_material(|m| m.color = color);
        penetration.update(app);
        Self {
            _position: position,
            _penetration: penetration,
        }
    }
}
