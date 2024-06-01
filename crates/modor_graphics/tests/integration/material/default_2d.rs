use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, DefaultMaterial2D, Mat, Model2D, Size, Texture, TextureGlob, TextureSource,
};
use modor_input::modor_math::Vec2;
use modor_resources::testing::wait_resource;
use modor_resources::Res;

#[modor::test(disabled(macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "material#white");
}

#[modor::test(disabled(macos, android, wasm))]
fn set_properties() {
    let (mut app, target) = configure_app();
    let texture = root(&mut app).texture.glob().clone();
    root(&mut app).material.texture = texture;
    root(&mut app).material.is_ellipse = true;
    root(&mut app).material.color = Color::DARK_GRAY;
    root(&mut app).material.texture_size = Vec2::ONE * 0.75;
    root(&mut app).material.texture_position = Vec2::ONE * 0.25;
    Root::wait_resources(&mut app);
    assert_same(&mut app, &target, "material#custom_default");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = app.get_mut::<Root>().target.glob().clone();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
    texture: Res<Texture>,
    material: Mat<DefaultMaterial2D>,
    model: Model2D<DefaultMaterial2D>,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut target =
            Res::<Texture>::from_source(ctx, "target", TextureSource::Size(Size::new(30, 20)));
        target.is_target_enabled = true;
        target.is_buffer_enabled = true;
        let mut texture =
            Res::<Texture>::from_path(ctx, "main", "../tests/assets/opaque-texture.png");
        texture.is_smooth = false;
        let material_data = DefaultMaterial2D::new(ctx);
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.size = Vec2::ONE * 0.5;
        model.camera = target.camera.glob().clone();
        Self {
            texture,
            material,
            model,
            target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.texture);
        wait_resource(app, |r: &Self| &r.target);
        app.update();
    }
}
