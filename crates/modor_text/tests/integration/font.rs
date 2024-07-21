use modor::log::Level;
use modor::{App, GlobRef, Node, RootNode, Visit};
use modor_graphics::modor_resources::testing::wait_resources;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::testing::assert_max_component_diff;
use modor_graphics::{Size, Texture, TextureGlob, TextureSource};
use modor_text::{Font, FontSource, Text2D};

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_ttf_font_from_path() {
    let (mut app, target) = configure_app();
    let font =
        Font::new(&mut app).load_from_path(&mut app, "../tests/assets/IrishGrover-Regular.ttf");
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#ttf", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_otf_font_from_path() {
    let (mut app, target) = configure_app();
    let font = Font::new(&mut app).load_from_path(&mut app, "../tests/assets/Foglihtenno07.otf");
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#otf", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_font_from_bytes() {
    let (mut app, target) = configure_app();
    let font = Font::new(&mut app).load_from_source(
        &mut app,
        FontSource::Bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/assets/Foglihtenno07.otf"
        ))),
    );
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#otf", 20, 2);
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.glob().to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn set_font(app: &mut App, font: Res<Font>) {
    root(app).text.font = font.glob().to_ref();
    root(app).font = Some(font);
}

#[derive(Node, Visit)]
struct Root {
    text: Text2D,
    target: Res<Texture>,
    font: Option<Res<Font>>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_buffer_enabled(true)
            .with_is_target_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(60, 40)));
        Self {
            text: Text2D::new(app)
                .with_content("text".into())
                .with_font_height(30.)
                .with_texture(|t| t.is_smooth = false)
                .with_model(|m| m.camera = target.camera.glob().to_ref()),
            target,
            font: None,
        }
    }
}
