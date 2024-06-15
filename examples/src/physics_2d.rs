use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::{Inputs, MouseButton};
use modor_graphics::{Color, Sprite2D, Window};
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
        Self {
            body: Body2D::new(ctx)
                .with_position(position)
                .with_size(size)
                .with_collision_group(Some(collision_group)),
            sprite: Sprite2D::new(ctx, "wall")
                .with_model(|m| m.position = position)
                .with_model(|m| m.size = size),
        }
    }
}

#[derive(Visit)]
struct Cannon {
    body: Body2D,
    sprite: Sprite2D,
}

impl Node for Cannon {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        let cursor_position = Self::cursor_position(ctx);
        self.body.rotation = Vec2::Y.rotation(cursor_position - CANNON_JOIN_POSITION);
        self.body.position =
            CANNON_JOIN_POSITION + (Vec2::Y * CANNON_LENGTH / 2.).with_rotation(self.body.rotation);
        self.sprite.model.position = self.body.position;
        self.sprite.model.rotation = self.body.rotation;
        Self::create_object(ctx, self.body.rotation);
    }
}

impl Cannon {
    fn new(ctx: &mut Context<'_>) -> Self {
        let size = Vec2::new(0.05, CANNON_LENGTH);
        Self {
            body: Body2D::new(ctx).with_size(size),
            sprite: Sprite2D::new(ctx, "cannon").with_model(|m| m.size = size),
        }
    }

    fn cursor_position(ctx: &mut Context<'_>) -> Vec2 {
        let inputs = ctx.get_mut::<Inputs>();
        let window_position = if let Some((_, finger)) = inputs.fingers.iter().next() {
            finger.position
        } else {
            inputs.mouse.position
        };
        let window = ctx.get_mut::<Window>();
        window.camera.world_position(window.size(), window_position)
    }

    fn create_object(ctx: &mut Context<'_>, rotation: f32) {
        let position = CANNON_JOIN_POSITION
            + (Vec2::Y * (CANNON_LENGTH + OBJECT_RADIUS / 2.)).with_rotation(rotation);
        let velocity = Vec2::Y.with_rotation(rotation) * OBJECT_INITIAL_SPEED;
        let object = match Self::object_type_to_create(ctx) {
            ObjectToCreate::Box => Some(Object::new(ctx, position, velocity, false)),
            ObjectToCreate::Ball => Some(Object::new(ctx, position, velocity, true)),
            ObjectToCreate::None => None,
        };
        ctx.get_mut::<Objects>().objects.extend(object);
    }

    fn object_type_to_create(ctx: &mut Context<'_>) -> ObjectToCreate {
        let inputs = ctx.get_mut::<Inputs>();
        if let Some((_, finger)) = inputs.fingers.iter().next() {
            if finger.state.is_just_released() {
                ObjectToCreate::Box
            } else {
                ObjectToCreate::None
            }
        } else if inputs.mouse[MouseButton::Left].is_just_released() {
            ObjectToCreate::Box
        } else if inputs.mouse[MouseButton::Right].is_just_released() {
            ObjectToCreate::Ball
        } else {
            ObjectToCreate::None
        }
    }
}

enum ObjectToCreate {
    Box,
    Ball,
    None,
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

#[derive(Visit)]
struct Object {
    body: Body2D,
    sprite: Sprite2D,
}

impl Node for Object {
    fn on_enter(&mut self, _ctx: &mut Context<'_>) {
        self.sprite.model.position = self.body.position;
        self.sprite.model.rotation = self.body.rotation;
    }
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
        Self {
            body: Body2D::new(ctx)
                .with_position(position)
                .with_size(size)
                .with_velocity(velocity)
                .with_force(-Vec2::Y * GRAVITY * OBJECT_MASS)
                .with_mass(OBJECT_MASS)
                .with_angular_inertia(OBJECT_MASS * OBJECT_RADIUS.powi(2) / inertia_factor)
                .with_collision_group(Some(collision_group))
                .with_shape(shape),
            sprite: Sprite2D::new(ctx, "object")
                .with_model(|m| m.size = size)
                .with_material(|m| m.is_ellipse = is_ball)
                .with_material(|m| m.color = color),
        }
    }
}
