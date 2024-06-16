use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::{Inputs, MouseButton};
use modor_graphics::{Color, CursorTracker, Sprite2D};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, CollisionGroup, CollisionType, Impulse, Shape2D};
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

#[derive(Node, Visit)]
struct Root {
    left_wall: Wall,
    right_wall: Wall,
    bottom_wall: Wall,
    cannon: Cannon,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            left_wall: Wall::new(ctx, Vec2::X * -0.5, Vec2::new(0.03, 1.)),
            right_wall: Wall::new(ctx, Vec2::X * 0.5, Vec2::new(0.03, 1.)),
            bottom_wall: Wall::new(ctx, Vec2::Y * -0.5, Vec2::new(1., 0.03)),
            cannon: Cannon::new(ctx),
        }
    }
}

#[derive(Node, Visit)]
struct CollisionGroups {
    wall: CollisionGroup,
    object: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let wall = CollisionGroup::new(ctx);
        let object = CollisionGroup::new(ctx);
        let impulse = CollisionType::Impulse(Impulse::new(0.1, 0.8));
        object.add_interaction(ctx, wall.glob(), impulse);
        object.add_interaction(ctx, object.glob(), impulse);
        Self { wall, object }
    }
}

#[derive(Node, Visit)]
struct Wall {
    body: Body2D,
    sprite: Sprite2D,
}

impl Wall {
    fn new(ctx: &mut Context<'_>, position: Vec2, size: Vec2) -> Self {
        let collision_group = ctx.get_mut::<CollisionGroups>().wall.glob().clone();
        let body = Body2D::new(ctx)
            .with_position(position)
            .with_size(size)
            .with_collision_group(Some(collision_group));
        let sprite = Sprite2D::new(ctx, "wall").with_model(|m| m.body = Some(body.glob().clone()));
        Self { body, sprite }
    }
}

#[derive(Visit)]
struct Cannon {
    sprite: Sprite2D,
    cursor: CursorTracker,
}

impl Node for Cannon {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let cursor_position = self.cursor.position(ctx);
        self.sprite.model.rotation = Vec2::Y.rotation(cursor_position - CANNON_JOIN_POSITION);
        self.sprite.model.position = CANNON_JOIN_POSITION
            + (Vec2::Y * CANNON_LENGTH / 2.).with_rotation(self.sprite.model.rotation);
        self.create_object(ctx, self.sprite.model.rotation);
    }
}

impl Cannon {
    fn new(ctx: &mut Context<'_>) -> Self {
        Self {
            sprite: Sprite2D::new(ctx, "cannon")
                .with_model(|m| m.size = Vec2::new(0.05, CANNON_LENGTH)),
            cursor: CursorTracker::new(ctx),
        }
    }

    fn create_object(&self, ctx: &mut Context<'_>, rotation: f32) {
        let position = CANNON_JOIN_POSITION
            + (Vec2::Y * (CANNON_LENGTH + OBJECT_RADIUS / 2.)).with_rotation(rotation);
        let velocity = Vec2::Y.with_rotation(rotation) * OBJECT_INITIAL_SPEED;
        let object = if self.cursor.state(ctx).is_just_released() {
            Some(Object::new(ctx, position, velocity, false))
        } else if ctx.get_mut::<Inputs>().mouse[MouseButton::Right].is_just_released() {
            Some(Object::new(ctx, position, velocity, true))
        } else {
            None
        };
        ctx.get_mut::<Objects>().objects.extend(object);
    }
}

#[derive(Default, RootNode, Visit)]
struct Objects {
    objects: Vec<Object>,
}

impl Node for Objects {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.objects.retain(|objects| objects.body.position.y > -5.);
    }
}

#[derive(Node, Visit)]
struct Object {
    body: Body2D,
    sprite: Sprite2D,
}

impl Object {
    fn new(ctx: &mut Context<'_>, position: Vec2, velocity: Vec2, is_ball: bool) -> Self {
        let collision_group = ctx.get_mut::<CollisionGroups>().object.glob().clone();
        let size = Vec2::ONE * OBJECT_RADIUS * 2.;
        let mut rng = rand::thread_rng();
        let color = Color::rgb(
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
            rng.gen_range(0.0..1.0),
        );
        let (inertia_factor, shape) = if is_ball {
            (CIRCLE_INERTIA_FACTOR, Shape2D::Circle)
        } else {
            (RECTANGLE_INERTIA_FACTOR, Shape2D::Rectangle)
        };
        let body = Body2D::new(ctx)
            .with_position(position)
            .with_size(size)
            .with_velocity(velocity)
            .with_force(-Vec2::Y * GRAVITY * OBJECT_MASS)
            .with_mass(OBJECT_MASS)
            .with_angular_inertia(OBJECT_MASS * OBJECT_RADIUS.powi(2) / inertia_factor)
            .with_collision_group(Some(collision_group))
            .with_shape(shape);
        let sprite = Sprite2D::new(ctx, "object")
            .with_model(|m| m.body = Some(body.glob().clone()))
            .with_material(|m| m.is_ellipse = is_ball)
            .with_material(|m| m.color = color);
        Self { body, sprite }
    }
}