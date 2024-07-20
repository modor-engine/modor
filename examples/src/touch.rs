use modor::log::Level;
use modor::{App, Node, RootNode, Visit};
use modor_graphics::modor_input::modor_math::Vec2;
use modor_graphics::modor_input::Inputs;
use modor_graphics::{Sprite2D, Window};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Default, RootNode, Visit)]
struct Root {
    fingers: Vec<Sprite2D>,
}

impl Node for Root {
    fn on_enter(&mut self, app: &mut App) {
        self.fingers = app
            .get_mut::<Inputs>()
            .fingers
            .pressed_iter()
            .map(|(_, f)| f.position)
            .collect::<Vec<_>>()
            .into_iter()
            .map(|finger_position| Self::finger_sprite(app, finger_position))
            .collect();
    }
}

impl Root {
    fn finger_sprite(app: &mut App, finger_position: Vec2) -> Sprite2D {
        let window = app.get_mut::<Window>();
        let window_size = window.size();
        let camera = window.camera.glob().clone();
        let position = camera.get(app).world_position(window_size, finger_position);
        Sprite2D::new(app, "finger")
            .with_model(|m| m.position = position)
            .with_model(|m| m.size = Vec2::ONE * 0.3)
            .with_material(|m| m.is_ellipse = true)
    }
}
