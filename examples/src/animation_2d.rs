use modor::{systems, App, BuiltEntity, Component, SingleRef};
use modor_graphics::{
    model_2d, window_target, Color, Material, Model2DMaterial, RenderTarget, Sprite, Texture,
    TextureAnimation, WINDOW_CAMERA_2D,
};
use modor_input::{Key, Keyboard};
use modor_math::Vec2;
use modor_physics::{Dynamics2D, Transform2D};
use modor_resources::ResKey;

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window())
        .with_entity(character())
        .run(modor_graphics::runner);
}

fn window() -> impl BuiltEntity + Sized {
    window_target().updated(|r: &mut RenderTarget| r.background_color = Color::DARK_GRAY)
}

fn character() -> impl BuiltEntity {
    let texture_key = ResKey::unique("character");
    let sprites = Direction::Down.stopped_sprites();
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Rectangle)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.15)
        .updated(|m: &mut Material| m.texture_key = Some(texture_key))
        .component(Dynamics2D::new())
        .component(Texture::from_path(texture_key, "slime.png"))
        .with(|t| t.is_smooth = false)
        .component(TextureAnimation::new(5, 9, sprites))
        .component(Character::default())
}

#[derive(Component, Default)]
struct Character {
    direction: Direction,
}

#[systems]
impl Character {
    #[run]
    fn update(
        &mut self,
        transform: &mut Transform2D,
        dynamics: &mut Dynamics2D,
        animation: &mut TextureAnimation,
        keyboard: SingleRef<'_, '_, Keyboard>,
    ) {
        dynamics.velocity = 0.2
            * keyboard.get().direction(
                Key::ArrowLeft,
                Key::ArrowRight,
                Key::ArrowUp,
                Key::ArrowDown,
            );
        self.direction = self.direction(dynamics.velocity);
        animation.sprites = self.direction.sprites(dynamics.velocity == Vec2::ZERO);
        transform.size.x = self.direction.size_x_sign() * transform.size.x.abs();
    }

    fn direction(&self, direction: Vec2) -> Direction {
        if direction.x < 0. {
            Direction::Left
        } else if direction.x > 0. {
            Direction::Right
        } else if direction.y > 0. {
            Direction::Up
        } else if direction.y < 0. {
            Direction::Down
        } else {
            self.direction
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Direction {
    Left,
    Right,
    Up,
    #[default]
    Down,
}

impl Direction {
    fn sprites(self, is_stopped: bool) -> Vec<Sprite> {
        if is_stopped {
            self.stopped_sprites()
        } else {
            self.moving_sprites()
        }
    }

    fn moving_sprites(self) -> Vec<Sprite> {
        let texture_y = self.texture_y();
        vec![
            Sprite::new(0, texture_y + 1),
            Sprite::new(1, texture_y + 1),
            Sprite::new(2, texture_y + 1),
            Sprite::new(3, texture_y + 1),
            Sprite::new(4, texture_y + 1),
            Sprite::new(0, texture_y + 2),
            Sprite::new(1, texture_y + 2),
        ]
    }

    fn stopped_sprites(self) -> Vec<Sprite> {
        let texture_y = self.texture_y();
        vec![Sprite::new(0, texture_y), Sprite::new(1, texture_y)]
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
