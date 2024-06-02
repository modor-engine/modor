use log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::testing::assert_same;
use modor_graphics::{
    Color, DefaultMaterial2D, Mat, Model2D, Size, Texture, TextureGlob, TextureSource,
};
use modor_resources::testing::wait_resource;
use modor_resources::Res;

#[modor::test(disabled(windows, macos, android, wasm))]
fn use_default_background() {
    let (mut app, target) = configure_app();
    assert_same(&mut app, &target, "target#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_background() {
    let (mut app, target) = configure_app();
    root(&mut app).target.target.background_color = Color::RED;
    app.update();
    assert_same(&mut app, &target, "target#other_background");
    assert_eq!(target.get(&app.ctx()).size, Size::new(30, 20));
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    Root::wait_resources(&mut app);
    let target = root(&mut app).target.target.glob().clone();
    assert_eq!(target.get(&app.ctx()).size, Size::new(30, 20));
    let target = root(&mut app).target.glob().clone();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

#[derive(Node, Visit)]
struct Root {
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
        let material_data = DefaultMaterial2D::new(ctx);
        let material = Mat::new(ctx, "main", material_data);
        let mut model = Model2D::new(ctx, material.glob());
        model.camera = target.camera.glob().clone();
        Self {
            material,
            model,
            target,
        }
    }
}

impl Root {
    fn wait_resources(app: &mut App) {
        wait_resource(app, |r: &Self| &r.target);
        app.update();
    }
}
