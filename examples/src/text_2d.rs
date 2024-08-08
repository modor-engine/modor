use instant::Instant;
use modor::log::Level;
use modor::{App, FromApp, Glob, State};
use modor_graphics::modor_resources::Res;
use modor_graphics::{Color, Sprite2D};
use modor_physics::modor_math::Vec2;
use modor_text::{Font, Text2D};
use std::time::Duration;

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    background: Sprite2D,
    text: Text2D,
    last_update: Instant,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let size = Vec2::new(1., 0.2);
        let font = app.get_mut::<Resources>().font.to_ref();
        Self {
            background: Sprite2D::new(app)
                .with_model(|m| m.size = size)
                .with_material(|m| m.color = Color::rgb(0.1, 0.1, 0.1)),
            text: Text2D::new(app)
                .with_content("Loading".into())
                .with_font(font)
                .with_font_height(300.)
                .with_material(|m| m.color = Color::GREEN)
                .with_model(|m| m.size = size)
                .with_model(|m| m.z_index = 1),
            last_update: Instant::now(),
        }
    }
}

impl State for Root {
    fn update(&mut self, app: &mut App) {
        if self.last_update.elapsed() > Duration::from_secs(1) {
            let new_text = match self.text.content.matches('.').count() {
                0 => "Loading.",
                1 => "Loading..",
                2 => "Loading...",
                _ => "Loading",
            };
            self.text.content = new_text.into();
            self.last_update = Instant::now();
        }
        self.background.update(app);
        self.text.update(app);
    }
}

#[derive(FromApp)]
struct Resources {
    font: Glob<Res<Font>>,
}

impl State for Resources {
    fn init(&mut self, app: &mut App) {
        self.font
            .updater()
            .path("IrishGrover-Regular.ttf")
            .apply(app);
    }
}
