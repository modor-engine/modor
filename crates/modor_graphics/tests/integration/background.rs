use modor::testing::TestApp;
use modor::{entity, App, Built, EntityBuilder, SingleMut};
use modor_graphics::{testing, BackgroundColor, Color, GraphicsModule, SurfaceSize};

struct BackgroundUpdater;

#[entity]
impl BackgroundUpdater {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self)
    }

    #[run]
    fn run(mut color: SingleMut<'_, BackgroundColor>) {
        color.r = color.g;
    }
}

#[test]
fn update_background() {
    let mut app: TestApp = App::new()
        .with_entity(GraphicsModule::build_windowless(SurfaceSize::new(300, 200)))
        .with_entity(BackgroundColor::build(Color::GREEN))
        .with_entity(BackgroundUpdater::build())
        .into();
    app.update();
    app.update();
    testing::assert_capture(&app, "tests/expected/background.png");
}
