use modor::{
    systems, App, BuiltEntity, Component, EntityBuilder, NoSystem, Query, Single,
    SingletonComponent,
};
use modor_graphics_new2::{
    Camera2D, Color, Font, FontSource, FrameRate, Material, Model, RenderTarget, Text,
    TextMaterialBuilder, Window,
};
use modor_input::{InputModule, Key, Keyboard, Mouse};
use modor_math::Vec2;
use modor_physics::{
    Collider2D, CollisionGroupRef, CollisionType, Dynamics2D, PhysicsModule, Transform2D,
};

const FIELD_POSITION: Vec2 = Vec2::new(0., -0.1);
const FIELD_SIZE: Vec2 = Vec2::new(1., 0.7);
const BALL_DIAMETER: f32 = 0.05;

fn main() {
    let vertical_wall_size = Vec2::new(0.01, FIELD_SIZE.y + 0.01);
    let horizontal_wall_size = Vec2::new(FIELD_SIZE.x + 0.01, 0.01);
    let field_left = FIELD_POSITION - Vec2::new(FIELD_SIZE.x / 2., 0.);
    let field_right = FIELD_POSITION + Vec2::new(FIELD_SIZE.x / 2., 0.);
    let field_top = FIELD_POSITION - Vec2::new(0., FIELD_SIZE.y / 2.);
    let field_bottom = FIELD_POSITION + Vec2::new(0., FIELD_SIZE.y / 2.);
    App::new()
        .with_entity(PhysicsModule::build())
        .with_entity(InputModule::build())
        .with_entity(modor_graphics_new2::renderer())
        .with_entity(FrameRate::VSync)
        .with_entity(Font::new(
            FontKey,
            FontSource::Path("LuckiestGuy-Regular.ttf".into()),
        ))
        .with_entity(window())
        .with_entity(Material::new(MaterialKey::White).with_color(Color::WHITE))
        .with_entity(Material::ellipse(MaterialKey::WhiteEllipse).with_color(Color::WHITE))
        .with_entity(left_score())
        .with_entity(right_score())
        .with_entity(wall(field_left, vertical_wall_size))
        .with_entity(wall(field_right, vertical_wall_size))
        .with_entity(wall(field_top, horizontal_wall_size))
        .with_entity(wall(field_bottom, horizontal_wall_size))
        .with_entity(middle_line())
        .with_entity(paddle(field_left + Vec2::X * 0.1).with(Paddle::left()))
        .with_entity(paddle(field_right - Vec2::X * 0.1).with(Paddle::right()))
        .with_entity(ball())
        .with_entity(cursor())
        .run(modor_graphics_new2::runner);
}

fn window() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(RenderTarget::new(TargetKey))
        .with(Window::default().with_cursor_shown(false))
        .with(Camera2D::new(CameraKey).with_target_key(TargetKey))
}

fn left_score() -> impl BuiltEntity {
    score(-0.25).with(LeftScore).with_inherited(
        TextMaterialBuilder::new(MaterialKey::LeftScore, "", 100.)
            .with_material(|m| {
                m.with_color(Color::INVISIBLE)
                    .with_front_color(Color::WHITE)
            })
            .with_text(|t| t.with_font(FontKey))
            .build(),
    )
}

fn right_score() -> impl BuiltEntity {
    score(0.25).with(RightScore).with_inherited(
        TextMaterialBuilder::new(MaterialKey::RightScore, "", 100.)
            .with_material(|m| {
                m.with_color(Color::INVISIBLE)
                    .with_front_color(Color::WHITE)
            })
            .with_text(|t| t.with_font(FontKey))
            .build(),
    )
}

fn score(x: f32) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(Vec2::new(x, 0.40))
                .with_size(Vec2::new(0.3, 0.1)),
        )
        .with(Model::rectangle(MaterialKey::LeftScore).with_camera_key(CameraKey))
        .with(Score(0))
}

fn wall(position: Vec2, size: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_position(position).with_size(size))
        .with(Model::rectangle(MaterialKey::White).with_camera_key(CameraKey))
        .with(Collider2D::rectangle(CollisionGroupKey::Wall))
}

fn middle_line() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(FIELD_POSITION)
                .with_size(Vec2::new(0.005, FIELD_SIZE.y)),
        )
        .with(Model::rectangle(MaterialKey::White).with_camera_key(CameraKey))
}

fn paddle(position: Vec2) -> impl BuiltEntity {
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(position)
                .with_size(Vec2::new(0.05, 0.2)),
        )
        .with(Dynamics2D::new())
        .with(Collider2D::rectangle(CollisionGroupKey::Paddle))
        .with(Model::rectangle(MaterialKey::White).with_camera_key(CameraKey))
}

fn ball() -> impl BuiltEntity {
    let velocity = Vec2::new(0.5, 0.).with_rotation(45_f32.to_radians());
    EntityBuilder::new()
        .with(
            Transform2D::new()
                .with_position(FIELD_POSITION)
                .with_size(Vec2::ONE * BALL_DIAMETER),
        )
        .with(Dynamics2D::new().with_velocity(velocity))
        .with(Collider2D::circle(CollisionGroupKey::Ball))
        .with(Model::rectangle(MaterialKey::WhiteEllipse).with_camera_key(CameraKey))
        .with(Ball)
}

fn cursor() -> impl BuiltEntity {
    EntityBuilder::new()
        .with(Transform2D::new().with_size(Vec2::ONE * 0.02))
        .with(Model::rectangle(MaterialKey::WhiteEllipse).with_camera_key(CameraKey))
        .with(Cursor)
}

#[derive(Component)]
struct Score(u32);

#[systems]
impl Score {
    #[run]
    fn update(&self, text: &mut Text) {
        text.content = format!("{}", self.0);
    }
}

#[derive(SingletonComponent, NoSystem)]
struct LeftScore;

#[derive(SingletonComponent, NoSystem)]
struct RightScore;

#[derive(Component)]
struct Paddle {
    up_key: Key,
    down_key: Key,
}

#[systems]
impl Paddle {
    fn left() -> Self {
        Self {
            up_key: Key::Z,
            down_key: Key::S,
        }
    }

    fn right() -> Self {
        Self {
            up_key: Key::Up,
            down_key: Key::Down,
        }
    }

    #[run]
    fn update(
        &self,
        transform: &mut Transform2D,
        dynamics: &mut Dynamics2D,
        keyboard: Single<'_, Keyboard>,
    ) {
        let min_y = FIELD_POSITION.y + FIELD_SIZE.y / 2. - transform.size.y / 2.;
        let max_y = FIELD_POSITION.y - FIELD_SIZE.y / 2. + transform.size.y / 2.;
        let velocity = keyboard.axis(self.down_key, self.up_key);
        dynamics.velocity.y = if transform.position.y >= min_y {
            transform.position.y = min_y;
            velocity.min(0.)
        } else if transform.position.y <= max_y {
            transform.position.y = max_y;
            velocity.max(0.)
        } else {
            velocity
        };
    }
}

#[derive(Component)]
struct Ball;

#[systems]
impl Ball {
    #[run]
    #[allow(clippy::cast_precision_loss)]
    fn update(transform: &mut Transform2D, dynamics: &mut Dynamics2D, collider: &Collider2D) {
        let collisions = collider
            .collisions()
            .iter()
            .filter(|c| {
                c.has_other_entity_group(CollisionGroupKey::Wall)
                    || c.has_other_entity_group(CollisionGroupKey::Paddle)
            })
            .collect::<Vec<_>>();
        if !collisions.is_empty() {
            let collision_normal = collisions
                .iter()
                .map(|c| c.normal)
                .fold(Vec2::ZERO, |a, b| a + b)
                / collisions.len() as f32;
            let collision_position = collisions
                .iter()
                .map(|c| c.position)
                .fold(Vec2::ZERO, |a, b| a + b)
                / collisions.len() as f32;
            *transform.position = collision_position - collision_normal * BALL_DIAMETER / 2.;
            *dynamics.velocity = -dynamics.velocity.mirror(collision_normal);
        }
    }
}

#[derive(Component)]
struct Cursor;

#[systems]
impl Cursor {
    #[run_after(
        component(InputModule),
        component(Mouse),
        component(Camera2D),
        component(Window)
    )]
    fn update(
        transform: &mut Transform2D,
        mouse: Single<'_, Mouse>,
        camera: Query<'_, &Camera2D>,
        window: Query<'_, &Window>,
    ) {
        let Some(camera) = camera.iter().next() else { return; };
        let Some(window) = window.iter().next() else { return; };
        *transform.position = camera.world_position(window, mouse.position());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TargetKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CameraKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    White,
    WhiteEllipse,
    LeftScore,
    RightScore,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CollisionGroupKey {
    Ball,
    Paddle,
    Wall,
}

impl CollisionGroupRef for CollisionGroupKey {
    fn collision_type(&self, other: &Self) -> CollisionType {
        match (self, other) {
            (Self::Ball, Self::Wall | Self::Paddle) => CollisionType::Sensor,
            _ => CollisionType::None,
        }
    }
}
