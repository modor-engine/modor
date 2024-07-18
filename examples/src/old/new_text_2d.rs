use std::time::{Duration, Instant};

const SIZE: Vec2 = Vec2::new(1., 0.2);
const SCOPE: Scope = Scope::new("root");

fn main() {
    modor_graphics::run(Level::Info, update);
}

fn update(app: &mut App) {
    update_background(app);
    update_label(app);
}

fn update_background(app: &mut App) {
    app.get_mut::<Sprite2D>(SCOPE.key(0))
        .set_size(SIZE)
        .set_color(Color::rgb(0.1, 0.1, 0.1));
}

fn update_label(app: &mut App) {
    app.get_mut::<Text2D>(SCOPE.key(0))
        .set_content(app.get_mut::<Label>(()).value())
        .set_font(Font::from_path("IrishGrover-Regular.ttf"))
        .set_font_height(300.)
        .set_color(Color::GREEN)
        .set_size(size)
        .set_z_index(1);
}

#[derive(Default, SingletonData)]
struct Label {
    last_update: Instant,
    value: &'static str,
}

impl Label {
    fn value(&mut self) -> &'static str {
        if self.last_update.elapsed() > Duration::from_secs(1) {
            let new_text = match label.value.matches('.').count() {
                0 => "Loading.",
                1 => "Loading..",
                2 => "Loading...",
                _ => "Loading",
            };
            self.value = new_text.into();
            self.last_update = Instant::now();
        }
        self.value
    }
}
