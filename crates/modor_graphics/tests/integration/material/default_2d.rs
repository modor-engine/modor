use log::Level;
use modor::{App, GlobRef, RootNode};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, DefaultMaterial2D, IntoMat, Mat, Model2D, Size, Texture, TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resources;
use modor_resources::{Res, ResLoad};

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    assert_same(&app, &target, "material#white");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_properties() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.glob().to_ref();
    root(&mut app).material.texture = texture;
    root(&mut app).material.is_ellipse = true;
    root(&mut app).material.color = Color::DARK_GRAY;
    root(&mut app).material.texture_size = Vec2::ONE * 0.75;
    root(&mut app).material.texture_position = Vec2::ONE * 0.25;
    wait_resources(&mut app);
    assert_same(&app, &target, "material#custom_default");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = app.get_mut::<Root>().target.glob().to_ref();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

struct Root {
    texture: Res<Texture>,
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(app: &mut App) -> Self {
        let target = Texture::new(app)
            .with_is_target_enabled(true)
            .with_is_buffer_enabled(true)
            .load_from_source(app, TextureSource::Size(Size::new(30, 20)));
        let texture = Texture::new(app)
            .with_is_smooth(false)
            .load_from_path(app, "../tests/assets/opaque-texture.png");
        let material = DefaultMaterial2D::new(app).into_mat(app);
        let model = Model2D::new(app, material.glob())
            .with_size(Vec2::ONE * 0.5)
            .with_camera(target.camera.glob().to_ref());
        Self {
            texture,
            material,
            model,
            target,
        }
    }

    fn update(&mut self, app: &mut App) {
        self.texture.update(app);
        self.material.update(app);
        self.model.update(app);
        self.target.update(app);
    }
}
