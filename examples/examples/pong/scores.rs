use crate::field;
use crate::Side;
use modor::{systems, BuiltEntity, SingletonComponent};
use modor_graphics::{Color, Material, WINDOW_CAMERA_2D};
use modor_math::Vec2;
use modor_physics::Transform2D;
use modor_text::{text_2d, Text};

pub(crate) fn score(side: Side) -> impl BuiltEntity {
    const TEXT_HEIGHT: f32 = 0.2;
    text_2d(WINDOW_CAMERA_2D, "0", 100.)
        .updated(|m: &mut Material| m.color = Color::INVISIBLE)
        .updated(|m: &mut Material| m.front_color = Color::WHITE)
        .updated(|t: &mut Transform2D| t.position.x = side.x_sign() * field::SIZE.x / 4.)
        .updated(|t: &mut Transform2D| t.position.y = field::SIZE.y / 2. - TEXT_HEIGHT / 2.)
        .updated(|t: &mut Transform2D| t.size = Vec2::new(0.3, TEXT_HEIGHT))
        .component_option((side == Side::Left).then_some(LeftScore(0)))
        .component_option((side == Side::Right).then_some(RightScore(0)))
}

#[derive(SingletonComponent, Debug)]
pub(crate) struct LeftScore(pub(crate) u32);

#[systems]
impl LeftScore {
    #[run]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.to_string();
    }
}

#[derive(SingletonComponent, Debug)]
pub(crate) struct RightScore(pub(crate) u32);

#[systems]
impl RightScore {
    #[run]
    fn update_display(&self, text: &mut Text) {
        text.content = self.0.to_string();
    }
}
