use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_input::{Inputs, Key};
use modor_graphics::modor_resources::{Res, ResUpdater};
use modor_graphics::{
    Color, Sprite2D, Texture, TextureAnimation, TexturePart, TextureUpdater, Window,
};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, Body2DUpdater};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root {
    slime: Slime,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.slime.init(app);
        app.get_mut::<Window>().target.background_color = Color::DARK_GRAY;
    }

    fn update(&mut self, app: &mut App) {
        self.slime.update(app);
    }
}

#[derive(FromApp)]
struct Resources {
    slime_texture: Glob<Res<Texture>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        TextureUpdater::default()
            .res(ResUpdater::default().path("slime.png"))
            .is_smooth(false)
            .apply(app, &self.slime_texture);
    }
}

struct Slime {
    body: Glob<Body2D>,
    sprite: Sprite2D,
    animation: TextureAnimation,
    direction: Direction,
}

impl FromApp for Slime {
    fn from_app(app: &mut App) -> Self {
        Self {
            body: Glob::from_app(app),
            sprite: Sprite2D::new(app),
            animation: TextureAnimation::new(5, 9),
            direction: Direction::Down,
        }
    }
}

impl Slime {
    fn init(&mut self, app: &mut App) {
        Body2DUpdater::default()
            .size(Vec2::ONE * 0.15)
            .apply(app, &self.body);
        self.sprite.model.body = Some(self.body.to_ref());
        self.sprite.material.texture = app.get_mut::<Resources>().slime_texture.to_ref();
        self.animation.parts = Direction::Down.stopped_texture_parts();
    }

    fn update(&mut self, app: &mut App) {
        let direction = app.get_mut::<Inputs>().keyboard.direction(
            Key::ArrowLeft,
            Key::ArrowRight,
            Key::ArrowUp,
            Key::ArrowDown,
        );
        self.direction = Direction::from_vec(direction).unwrap_or(self.direction);
        self.animation.parts = self.direction.texture_parts(direction == Vec2::ZERO);
        self.sprite.material.texture_size = self.animation.part_size();
        self.sprite.material.texture_position = self.animation.part_position();
        Body2DUpdater::default()
            .for_size(|s| s.x = self.direction.size_x_sign() * s.x.abs())
            .velocity(0.2 * direction)
            .apply(app, &self.body);
        self.sprite.update(app);
        self.animation.update(app);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
