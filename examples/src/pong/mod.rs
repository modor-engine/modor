use crate::pong::ball::Ball;
use crate::pong::paddle::Paddle;
use crate::pong::scores::Scores;
use crate::pong::wall::{FIELD_BORDER_WIDTH, FIELD_SIZE};
use collisions::CollisionGroups;
use modor::log::Level;
use modor::{App, FromApp, State};
use modor_graphics::Sprite2D;
use modor_physics::modor_math::Vec2;
use side::Side;
use wall::{Wall, WallOrientation};

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(FromApp)]
struct Root {
    left_wall: Wall,
    right_wall: Wall,
    top_wall: Wall,
    bottom_wall: Wall,
    separator: Sprite2D,
    ball: Ball,
    left_paddle: Paddle,
    right_paddle: Paddle,
}

impl State for Root {
    fn init(&mut self, app: &mut App) {
        app.create::<Scores>();
        app.take::<CollisionGroups, _>(|groups, app| {
            self.left_wall
                .init(app, WallOrientation::Left, &groups.vertical_wall);
            self.right_wall
                .init(app, WallOrientation::Right, &groups.vertical_wall);
            self.top_wall
                .init(app, WallOrientation::Top, &groups.horizontal_wall);
            self.bottom_wall
                .init(app, WallOrientation::Bottom, &groups.horizontal_wall);
        });
        self.separator.model.size = Vec2::new(FIELD_BORDER_WIDTH / 4., FIELD_SIZE.y);
        self.ball.init(app);
        self.left_paddle.init_player(app, Side::Left);
        self.right_paddle.init_bot(app, Side::Right);
    }

    fn update(&mut self, app: &mut App) {
        self.left_wall.update(app);
        self.right_wall.update(app);
        self.top_wall.update(app);
        self.bottom_wall.update(app);
        self.separator.update(app);
        self.ball.update(app);
        self.left_paddle.update(app);
        self.right_paddle.update(app);
    }
}

mod ball;
mod collisions;
mod paddle;
mod scores;
mod side;
mod wall;
