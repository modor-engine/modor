use modor::log::Level;
use modor::{App, Context, GlobRef, Node, RootNode, Visit};
use modor_graphics::modor_resources::testing::wait_resources;
use modor_graphics::modor_resources::{Res, ResLoad};
use modor_graphics::testing::assert_same;
use modor_graphics::{Size, Texture, TextureGlob, TextureSource};
use modor_text::{Alignment, Text2D};

#[modor::test(disabled(windows, macos, android, wasm))]
fn create_default() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    assert_same(&mut app, &target, "text#default");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn set_content() {
    let (mut app, target) = configure_app();
    wait_resources(&mut app);
    text(&mut app).content = "Content".into();
    app.update();
    assert_same(&mut app, &target, "text#other_content");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_left_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Left;
    wait_resources(&mut app);
    assert_same(&mut app, &target, "text#left_alignment");
}

#[modor::test(disabled(windows, macos, android, wasm))]
fn apply_right_alignment() {
    let (mut app, target) = configure_app();
    text(&mut app).alignment = Alignment::Right;
    wait_resources(&mut app);
    assert_same(&mut app, &target, "text#right_alignment");
}

fn configure_app() -> (App, GlobRef<TextureGlob>) {
    let mut app = App::new::<Root>(Level::Info);
    let target = root(&mut app).target.glob().clone();
    (app, target)
}

fn root(app: &mut App) -> &mut Root {
    app.get_mut::<Root>()
}

fn text(app: &mut App) -> &mut Text2D {
    &mut root(app).text
}

#[derive(Node, Visit)]
struct Root {
    text: Text2D,
    target: Res<Texture>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let target = Texture::new(ctx, "target")
            .with_is_buffer_enabled(true)
            .with_is_target_enabled(true)
            .load_from_source(ctx, TextureSource::Size(Size::new(100, 50)));
        Self {
            text: Text2D::new(ctx, "main")
                .with_content("text\nto\nrender".into())
                .with_texture(|t| t.is_smooth = false)
                .with_model(|m| m.camera = target.camera.glob().clone()),
            target,
        }
    }
}
