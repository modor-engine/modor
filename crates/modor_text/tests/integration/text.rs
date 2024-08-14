use modor::log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::modor_resources::testing::wait_resources;
use modor_graphics::modor_resources::{Res, ResUpdater};
use modor_graphics::testing::assert_max_component_diff;
use modor_graphics::{Size, Texture, TextureSource, TextureUpdater};
use modor_text::{Alignment, Text2D};

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "text#default", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_content() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    text(&mut app).content = "Content".into();
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "text#other_content", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_left_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Left;
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "text#left_alignment", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_right_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Right;
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "text#right_alignment", 20, 2);
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn text(app: &mut App) -> &mut Text2D {
    &mut root(app).text
}

struct Root {
    text: Text2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            text: Text2D::new(app),
            target: Glob::from_app(app),
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.text.content = "text\nto\nrender".into();
        self.text.model.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .is_smooth(false)
            .apply(app, &self.text.texture);
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(100, 50))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.text.update(app);
    }
}
