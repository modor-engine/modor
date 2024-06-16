use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::Inputs;
use modor_graphics::{Color, Sprite2D, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Collision2D, CollisionGroup, CollisionType, Shape2D};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_8};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    rectangle: Shape,
    circle: Shape,
    cursor: Cursor,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Window>().is_cursor_visible = false;
        Self {
            rectangle: Shape::new(ctx, Vec2::X * 0.25, Vec2::new(0.2, 0.3), false),
            circle: Shape::new(ctx, -Vec2::X * 0.25, Vec2::ONE * 0.4, true),
            cursor: Cursor::new(ctx),
        }
    }
}

#[derive(Node, Visit)]
struct CollisionGroups {
    shape: CollisionGroup,
    cursor: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let shape = CollisionGroup::new(ctx);
        let cursor = CollisionGroup::new(ctx);
        cursor.add_interaction(ctx, shape.glob(), CollisionType::Sensor);
        Self { shape, cursor }
    }
}

#[derive(Visit)]
struct Shape {
    body: Body2D,
    sprite: Sprite2D,
    collision: Vec<CollisionNormal>,
}

impl Node for Shape {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        self.collision.clear();
        for collision in self.body.collisions() {
            self.collision
                .push(CollisionNormal::new(ctx, collision, false));
        }
    }
}

impl Shape {
    fn new(ctx: &mut Context<'_>, position: Vec2, size: Vec2, is_circle: bool) -> Self {
        let collision_group = ctx.get_mut::<CollisionGroups>().shape.glob().clone();
        let body = Body2D::new(ctx)
            .with_position(position)
            .with_size(size)
            .with_collision_group(Some(collision_group))
            .with_shape(if is_circle {
                Shape2D::Circle
            } else {
                Shape2D::Rectangle
            });
        let sprite = Sprite2D::new(ctx, "shape")
            .with_model(|m| m.body = Some(body.glob().clone()))
            .with_material(|m| m.is_ellipse = is_circle)
            .with_material(|m| m.color = Color::CYAN);
        Self {
            body,
            sprite,
            collision: vec![],
        }
    }
}

#[derive(Visit)]
struct Cursor {
    body: Body2D,
    sprite: Sprite2D,
    collision: Vec<CollisionNormal>,
}

impl Node for Cursor {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let window_position = Self::window_position(ctx);
        let window = ctx.get_mut::<Window>();
        self.body.position = window.camera.world_position(window.size(), window_position);
        self.sprite.material.color = if self.body.collisions().is_empty() {
            Color::GREEN
        } else {
            Color::RED
        };
        self.collision.clear();
        for collision in self.body.collisions() {
            self.collision
                .push(CollisionNormal::new(ctx, collision, true));
        }
    }
}

impl Cursor {
    fn new(ctx: &mut Context<'_>) -> Self {
        let collision_group = ctx.get_mut::<CollisionGroups>().cursor.glob().clone();
        let body = Body2D::new(ctx)
            .with_size(Vec2::new(0.05, 0.1))
            .with_rotation(FRAC_PI_8)
            .with_collision_group(Some(collision_group));
        let sprite = Sprite2D::new(ctx, "cursor")
            .with_model(|m| m.body = Some(body.glob().clone()))
            .with_model(|m| m.rotation = FRAC_PI_8)
            .with_model(|m| m.z_index = 1)
            .with_material(|m| m.color = Color::GREEN);
        Self {
            body,
            sprite,
            collision: vec![],
        }
    }

    fn window_position(ctx: &mut Context<'_>) -> Vec2 {
        let inputs = ctx.get_mut::<Inputs>();
        if let Some((_, finger)) = inputs.fingers.iter().next() {
            finger.position
        } else {
            inputs.mouse.position
        }
    }
}

#[derive(Node, Visit)]
struct CollisionNormal {
    position: Sprite2D,
    penetration: Sprite2D,
}

impl CollisionNormal {
    fn new(ctx: &mut Context<'_>, collision: &Collision2D, from_cursor: bool) -> Self {
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
            position: Sprite2D::new(ctx, "collision-position")
                .with_model(|m| m.position = collision.position)
                .with_model(|m| m.size = Vec2::ONE * 0.02)
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color)
                .with_material(|m| m.is_ellipse = true),
            penetration: Sprite2D::new(ctx, "collision-penetration")
                .with_model(|m| m.position = penetration_position)
                .with_model(|m| m.size = Vec2::new(0.005, collision.penetration.magnitude()))
                .with_model(|m| m.rotation = Vec2::Y.rotation(-collision.penetration))
                .with_model(|m| m.z_index = z_index)
                .with_material(|m| m.color = color),
        }
    }
}
