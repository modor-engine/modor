use modor::{App, RootNode};
use modor_physics::{CollisionGroup, CollisionType, Impulse};

pub(crate) struct CollisionGroups {
    pub(crate) horizontal_wall: CollisionGroup,
    pub(crate) vertical_wall: CollisionGroup,
    pub(crate) paddle: CollisionGroup,
    pub(crate) ball: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(app: &mut App) -> Self {
        let horizontal_wall = CollisionGroup::new(app);
        let vertical_wall = CollisionGroup::new(app);
        let paddle = CollisionGroup::new(app);
        let impulse = Impulse::new(0., 0.);
        paddle.add_interaction(app, horizontal_wall.glob(), CollisionType::Impulse(impulse));
        let ball = CollisionGroup::new(app);
        let impulse = Impulse::new(1., 0.);
        ball.add_interaction(app, horizontal_wall.glob(), CollisionType::Impulse(impulse));
        ball.add_interaction(app, vertical_wall.glob(), CollisionType::Sensor);
        ball.add_interaction(app, paddle.glob(), CollisionType::Sensor);
        Self {
            horizontal_wall,
            vertical_wall,
            paddle,
            ball,
        }
    }

    fn update(&mut self, app: &mut App) {
        self.horizontal_wall.update(app);
        self.vertical_wall.update(app);
        self.paddle.update(app);
        self.ball.update(app);
    }
}
