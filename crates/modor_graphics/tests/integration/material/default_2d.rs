use log::Level;
use modor::{App, FromApp, Glob, GlobRef, State};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, DefaultMaterial2D, DefaultMaterial2DUpdater, MatGlob, Model2D, Size, Texture,
    TextureSource, TextureUpdater,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResUpdater};

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
    wait_resources(&mut app);
    app.take::<Root, _>(|root, app| {
        DefaultMaterial2DUpdater::default()
            .texture(texture)
            .is_ellipse(true)
            .color(Color::DARK_GRAY)
            .texture_size(Vec2::ONE * 0.75)
            .texture_position(Vec2::ONE * 0.25)
            .apply(app, &root.material);
    });
    app.update();
    assert_same(&app, &target, "material#custom_default");
    app.take::<Root, _>(|root, app| {
        DefaultMaterial2DUpdater::default()
            .for_texture(|_| ())
            .for_is_ellipse(|e| *e = false)
            .for_color(|c| *c = Color::WHITE)
            .for_texture_size(|s| *s = Vec2::ONE * 0.25)
            .for_texture_position(|p| *p = Vec2::ZERO)
            .apply(app, &root.material);
    });
    app.update();
    assert_same(&app, &target, "material#red");
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
    material: MatGlob<DefaultMaterial2D>,
    model: Model2D,
    target: Glob<Res<Texture>>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
        let target = Glob::from_app(app);
        let texture = Glob::from_app(app);
        let material = MatGlob::from_app(app);
        let model = Model2D::new(app).with_material(material.to_ref());
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
        TextureUpdater::default()
            .res(ResUpdater::default().path("../tests/assets/opaque-texture.png"))
            .is_smooth(false)
            .apply(app, &self.texture);
        self.model.size = Vec2::ONE * 0.5;
        self.model.camera = self.target.get(app).camera().glob().to_ref();
        TextureUpdater::default()
            .res(ResUpdater::default().source(TextureSource::Size(Size::new(30, 20))))
            .is_target_enabled(true)
            .is_buffer_enabled(true)
            .apply(app, &self.target);
    }

    fn update(&mut self, app: &mut App) {
        self.model.update(app);
    }
}
