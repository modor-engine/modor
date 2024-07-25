use modor::log::Level;
use modor::{App, GlobRef, RootNode};
use modor_graphics::modor_resources::testing::wait_resources;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::testing::assert_max_component_diff;
use modor_graphics::{Size, Texture, TextureGlob, TextureSource};
use modor_text::{Alignment, Text2D};

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    assert_max_component_diff(&app, &target, "text#default", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_content() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    text(&mut app).content = "Content".into();
    app.update();
    assert_max_component_diff(&app, &target, "text#other_content", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_left_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Left;
    wait_resources(&mut app);
    assert_max_component_diff(&app, &target, "text#left_alignment", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_right_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Right;
    wait_resources(&mut app);
    assert_max_component_diff(&app, &target, "text#right_alignment", 20, 2);
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.glob().to_ref();
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
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_buffer_enabled(true)
            .with_is_target_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(100, 50)));
        Self {
            text: Text2D::new(app)
                .with_content("text\nto\nrender".into())
                .with_texture(|t| t.is_smooth = false)
                .with_model(|m| m.camera = target.camera.glob().to_ref()),
            target,
        }
    }

    fn update(&mut self, app: &mut App) {
        self.text.update(app);
        self.target.update(app);
    }
}
