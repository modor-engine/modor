use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::{Color, Sprite2D, Texture, TextureAnimation, TexturePart, Window};
use modor_physics::modor_math::Vec2;
use modor_physics::Body2D;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Node, Visit)]
struct Root {
    slime: Slime,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.get_mut::<Window>().target.background_color = Color::DARK_GRAY;
        Self {
            slime: Slime::new(ctx),
        }
    }
}

#[derive(Node, Visit)]
struct Resources {
    smile_texture: Res<Texture>,
}

impl RootNode for Resources {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        Self {
            smile_texture: Texture::new(ctx, "slime")
                .with_is_smooth(false)
                .load_from_path(ctx, "slime.png"),
        }
    }
}

#[derive(Visit)]
struct Slime {
    body: Body2D,
    sprite: Sprite2D,
    animation: TextureAnimation,
    direction: Direction,
}

impl Node for Slime {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        self.body.velocity = 0.2
            * ctx.get_mut::<Inputs>().keyboard.direction(
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::ArrowUp,
                Key::ArrowDown,
            );
        self.direction = Direction::from_vec(self.body.velocity).unwrap_or(self.direction);
        self.body.size.x = self.direction.size_x_sign() * self.body.size.x.abs();
        let is_stopped = self.body.velocity == Vec2::ZERO;
        self.animation.parts = self.direction.texture_parts(is_stopped);
        self.sprite.material.texture_size = self.animation.part_size();
        self.sprite.material.texture_position = self.animation.part_position();
    }
}

impl Slime {
    fn new(ctx: &mut Context<'_>) -> Self {
        let texture = ctx.get_mut::<Resources>().smile_texture.glob().clone();
        let body = Body2D::new(ctx).with_size(Vec2::ONE * 0.15);
        let sprite = Sprite2D::new(ctx, "slime")
            .with_model(|m| m.body = Some(body.glob().clone()))
            .with_material(|m| m.texture = texture);
        Self {
            body,
            sprite,
            animation: TextureAnimation::new(5, 9)
                .with_parts(|p| *p = Direction::Down.stopped_texture_parts()),
            direction: Direction::Down,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    fn from_vec(direction: Vec2) -> Option<Self> {
        if direction.x < 0. {
            Some(Self::Left)
        } else if direction.x > 0. {
            Some(Self::Right)
        } else if direction.y > 0. {
            Some(Self::Up)
        } else if direction.y < 0. {
            Some(Self::Down)
        } else {
            None
        }
    }

    fn texture_parts(self, is_stopped: bool) -> Vec<TexturePart> {
        if is_stopped {
            self.stopped_texture_parts()
        } else {
            self.moving_texture_parts()
        }
    }

    fn moving_texture_parts(self) -> Vec<TexturePart> {
        let texture_y = self.texture_y();
        vec![
            TexturePart::new(0, texture_y + 1),
            TexturePart::new(1, texture_y + 1),
            TexturePart::new(2, texture_y + 1),
            TexturePart::new(3, texture_y + 1),
            TexturePart::new(4, texture_y + 1),
            TexturePart::new(0, texture_y + 2),
            TexturePart::new(1, texture_y + 2),
        ]
    }

    fn stopped_texture_parts(self) -> Vec<TexturePart> {
        let texture_y = self.texture_y();
        vec![
            TexturePart::new(0, texture_y),
            TexturePart::new(1, texture_y),
        ]
    }

    fn texture_y(self) -> u16 {
        match self {
            Self::Left | Self::Right => 0,
            Self::Up => 6,
            Self::Down => 3,
        }
    }

    fn size_x_sign(self) -> f32 {
        match self {
            Self::Left | Self::Up | Self::Down => 1.,
            Self::Right => -1.,
        }
    }
}
