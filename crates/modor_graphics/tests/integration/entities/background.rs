use modor::{App, With};
use modor_graphics::{testing, BackgroundColor, Capture, Color, GraphicsModule, SurfaceSize};

#[test]
fn update_background() {
    App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::from(Color::GREEN))
        .with_update::<(), _>(|c: &mut BackgroundColor| c.r = c.g)
        .updated()
        .assert::<With<Capture>>(1, |e| {
            testing::assert_capture(e, "tests/expected/background.png")
        });
}
