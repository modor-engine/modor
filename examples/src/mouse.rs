use modor::{systems, App, BuiltEntity, Single, SingleRef, SingletonComponent};
use modor_graphics::{
    model_2d, window_target, Camera2D, Color, Material, Model2DMaterial, Window, WINDOW_CAMERA_2D,
};
use modor_input::Mouse;
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Text};

pub fn main() {
    App::new()
        .with_entity(modor_text::module())
        .with_entity(window_target())
        .with_entity(cursor())
        .with_entity(text(0.25, "Pressed buttons:"))
        .with_entity(text(-0.25, "").component(PressedButtons::default()))
        .run(modor_graphics::runner);
}

fn cursor() -> impl BuiltEntity {
    model_2d(WINDOW_CAMERA_2D, Model2DMaterial::Ellipse)
        .updated(|t: &mut Transform2D| t.size = Vec2::ONE * 0.02)
        .component(CursorPosition::default())
}

fn text(position_y: f32, text: &str) -> impl BuiltEntity {
    text_2d(WINDOW_CAMERA_2D, text.to_string(), 50.)
        .updated(|t: &mut Transform2D| t.position = Vec2::Y * position_y)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(1., 0.15))
        .updated(|t: &mut Material| t.color = Color::INVISIBLE)
        .updated(|t: &mut Material| t.front_color = Color::WHITE)
}

#[derive(SingletonComponent, Default)]
struct CursorPosition(Vec2);

#[systems]
impl CursorPosition {
    #[run_after(component(Mouse))]
    fn retrieve(
        &mut self,
        mouse: SingleRef<'_, '_, Mouse>,
        window_camera: Single<'_, Window, (&Window, &Camera2D)>,
    ) {
        let (window, camera) = window_camera.get();
        self.0 = camera.world_position(window.size(), mouse.get().position);
    }

    #[run_after_previous]
    fn update_display(&self, transform: &mut Transform2D) {
        transform.position = self.0;
    }
}

#[derive(SingletonComponent, Default)]
struct PressedButtons(Vec<String>);

#[systems]
impl PressedButtons {
    #[run_after(component(Mouse))]
    fn retrieve(&mut self, mouse: SingleRef<'_, '_, Mouse>) {
        self.0.clear();
        for button in mouse.get().pressed_iter() {
            self.0.push(format!("{button:?}"));
        }
    }

    #[run_after_previous]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.join(", ");
    }
}
