use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State, Updater};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Size, Texture, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "material#white");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_properties() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.to_ref();
    root(&mut app).material.texture = texture;
    root(&mut app).material.is_ellipse = true;
    root(&mut app).material.color = Color::DARK_GRAY;
    root(&mut app).material.texture_size = Vec2::ONE * 0.75;
    root(&mut app).material.texture_position = Vec2::ONE * 0.25;
    wait_resources(&mut app);
    app.update();
    assert_same(&app, &target, "material#custom_default");
}

fn configure_app() -> (App, GlobRef<Res<Texture>>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = app.get_mut::<Root>().target.to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Glob<Res<Texture>>,
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let texture = Glob::from_app(app);
        let material = DefaultMaterial2D::new(app).into_mat(app);
        let model = Model2D::new(app, material.glob());
        Self {
            texture,
            material,
            model,
            target,
        }
    }
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        self.texture
            .updater()
            .path("../tests/assets/opaque-texture.png")
            .for_inner(app, |inner, app| {
                inner.updater().is_smooth(false).apply(app)
            })
            .apply(app);
        self.model.size = Vec2::ONE * 0.5;
        self.model.camera = self.target.get(app).camera.glob().to_ref();
        self.target
            .updater()
            .source(TextureSource::Size(Size::new(30, 20)))
            .for_inner(app, |inner, app| {
                inner
                    .updater()
                    .is_target_enabled(true)
                    .is_buffer_enabled(true)
                    .apply(app)
            })
            .apply(app);
    }

    fn update(&mut self, app: &mut App) {
        self.material.update(app);
        self.model.update(app);
    }
}
