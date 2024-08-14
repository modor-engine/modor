use modor::log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::modor_resources::testing::wait_resources;
use modor_graphics::modor_resources::{Res, ResUpdater};
use modor_graphics::testing::assert_max_component_diff;
use modor_graphics::{Size, Texture, TextureSource, TextureUpdater};
use modor_text::{Font, FontSource, FontUpdater, Text2D};

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_ttf_font_from_path() {
    let (mut app, target) = configure_app();
    let font = Glob::<Res<Font>>::from_app(&mut app);
    FontUpdater::default()
        .res(ResUpdater::default().path("../tests/assets/IrishGrover-Regular.ttf"))
        .apply(&mut app, &font);
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#ttf", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_otf_font_from_path() {
    let (mut app, target) = configure_app();
    let font = Glob::<Res<Font>>::from_app(&mut app);
    FontUpdater::default()
        .res(ResUpdater::default().path("../tests/assets/Foglihtenno07.otf"))
        .apply(&mut app, &font);
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#otf", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn render_font_from_bytes() {
    let (mut app, target) = configure_app();
    let font = Glob::<Res<Font>>::from_app(&mut app);
    FontUpdater::default()
        .res(
            ResUpdater::default().source(FontSource::Bytes(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/assets/Foglihtenno07.otf"
            )))),
        )
        .apply(&mut app, &font);
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#otf", 20, 2);
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_source() {
    let (mut app, target) = configure_app();
    let font = Glob::<Res<Font>>::from_app(&mut app);
    let font_ref = font.to_ref();
    FontUpdater::default()
        .res(ResUpdater::default().path("../tests/assets/IrishGrover-Regular.ttf"))
        .apply(&mut app, &font_ref);
    set_font(&mut app, font);
    wait_resources(&mut app);
    app.update();
    assert_max_component_diff(&app, &target, "font#ttf", 20, 2);
    FontUpdater::default()
        .res(ResUpdater::default().path("../tests/assets/Foglihtenno07.otf"))
        .apply(&mut app, &font_ref);
    wait_resources(&mut app);
    app.update();
    app.update();
    app.update();
    assert_max_component_diff(&app, &target, "font#otf", 20, 2);
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn set_font(app: &mut App, font: Glob<Res<Font>>) {
    root(app).text.font = font.to_ref();
    root(app).font = Some(font);
}

struct Root {
    text: Text2D,
    target: Glob<Res<Texture>>,
    font: Option<Glob<Res<Font>>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        Self {
            text: Text2D::new(app),
            target: Glob::from_app(app),
            font: None,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.text.content = "text".into();
        self.text.font_height = 30.;
        self.text.model.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .is_smooth(false)
            .apply(app, &self.text.texture);
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(60, 40))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.text.update(app);
    }
}
