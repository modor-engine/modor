use modor::log::Level;
use modor::{App, Node, RootNode};
use modor_graphics::{Color, CursorTracker, Sprite2D};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Collision2D, CollisionGroup, CollisionType, Shape2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    rectangle: Shape,
    circle: Shape,
    cursor: Cursor,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        // app.get_mut::<Window>().is_cursor_visible = false;
        Self {
            rectangle: Shape::new(app, Vec2::X * 0.25, Vec2::new(0.2, 0.3), false),
            circle: Shape::new(app, -Vec2::X * 0.25, Vec2::ONE * 0.4, true),
            cursor: Cursor::new(app),
        }
    }
}

impl Node for Root {
    fn update(&mut self, app: &mut App) {
        self.rectangle.update(app);
        self.circle.update(app);
        self.cursor.update(app);
    }
}

struct CollisionGroups {
    shape: CollisionGroup,
    cursor: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(app: &mut App) -> Self {
        let shape = CollisionGroup::new(app);
        let cursor = CollisionGroup::new(app);
        cursor.add_interaction(app, shape.glob(), CollisionType::Sensor);
        Self { shape, cursor }
    }
}

impl Node for CollisionGroups {
    fn update(&mut self, app: &mut App) {
        self.shape.update(app);
        self.cursor.update(app);
    }
}

struct Shape {
    body: Body2D,
    sprite: Sprite2D,
    collisions: Vec<CollisionNormal>,
}

impl Node for Shape {
    fn update(&mut self, app: &mut App) {
        self.collisions.clear();
        for collision in self.body.collisions() {
            self.collisions
                .push(CollisionNormal::new(app, collision, false));
        }
        self.body.update(app);
        self.sprite.update(app);
        for collision in &mut self.collisions {
            collision.update(app);
        }
    }
}

impl Shape {
    fn new(app: &mut App, position: Vec2, size: Vec2, is_circle: bool) -> Self {
        let collision_group = app.get_mut::<CollisionGroups>().shape.glob().to_ref();
        let body = Body2D::new(app)
            .with_position(position)
            .with_size(size)
            .with_collision_group(Some(collision_group))
            .with_shape(if is_circle {
                Shape2D::Circle
            } else {
                Shape2D::Rectangle
            });
        let sprite = Sprite2D::new(app)
            .with_model(|m| m.body = Some(body.glob().to_ref()))
            .with_material(|m| m.is_ellipse = is_circle)
            .with_material(|m| m.color = Color::CYAN);
        Self {
            body,
            sprite,
            collisions: vec![],
        }
    }
}

struct Cursor {
    body: Body2D,
    sprite: Sprite2D,
    collisions: Vec<CollisionNormal>,
    tracker: CursorTracker,
}

impl Node for Cursor {
    fn update(&mut self, app: &mut App) {
        self.tracker.update(app);
        self.body.position = self.tracker.position(app);
        self.sprite.material.color = if self.body.collisions().is_empty() {
            Color::GREEN
        } else {
            Color::RED
        };
        self.collisions.clear();
        for collision in self.body.collisions() {
            self.collisions
                .push(CollisionNormal::new(app, collision, true));
        }
        self.body.update(app);
        self.sprite.update(app);
        for collision in &mut self.collisions {
            collision.update(app);
        }
    }
}

impl Cursor {
    fn new(app: &mut App) -> Self {
        let collision_group = app.get_mut::<CollisionGroups>().cursor.glob().to_ref();
        let body = Body2D::new(app)
            .with_size(Vec2::new(0.05, 0.1))
            .with_rotation(FRAC_PI_8)
            .with_collision_group(Some(collision_group));
        let sprite = Sprite2D::new(app)
            .with_model(|m| m.body = Some(body.glob().to_ref()))
            .with_model(|m| m.rotation = FRAC_PI_8)
            .with_model(|m| m.z_index = 1)
            .with_material(|m| m.color = Color::GREEN);
        Self {
            body,
            sprite,
            collisions: vec![],
            tracker: CursorTracker::new(app),
        }
    }
}

struct CollisionNormal {
    position: Sprite2D,
    penetration: Sprite2D,
}

impl Node for CollisionNormal {
    fn update(&mut self, app: &mut App) {
        self.position.update(app);
        self.penetration.update(app);
    }
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
        Self {
            position: Sprite2D::new(app)
                .with_model(|m| m.position = collision.position)
                .with_model(|m| m.size = Vec2::ONE * 0.02)
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color)
                .with_material(|m| m.is_ellipse = true),
            penetration: Sprite2D::new(app)
                .with_model(|m| m.position = penetration_position)
                .with_model(|m| m.size = Vec2::new(0.005, collision.penetration.magnitude()))
                .with_model(|m| m.rotation = Vec2::Y.rotation(-collision.penetration))
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color),
        }
    }
}
