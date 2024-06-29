use modor::{Context, Node, RootNode, Visit};
use modor_physics::{CollisionGroup, CollisionType, Impulse};

#[derive(Node, Visit)]
pub(crate) struct CollisionGroups {
    pub(crate) horizontal_wall: CollisionGroup,
    pub(crate) vertical_wall: CollisionGroup,
    pub(crate) paddle: CollisionGroup,
    pub(crate) ball: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let horizontal_wall = CollisionGroup::new(ctx);
        let vertical_wall = CollisionGroup::new(ctx);
        let paddle = CollisionGroup::new(ctx);
        let impulse = Impulse::new(0., 0.);
        paddle.add_interaction(ctx, horizontal_wall.glob(), CollisionType::Impulse(impulse));
        let ball = CollisionGroup::new(ctx);
        let impulse = Impulse::new(1., 0.);
        ball.add_interaction(ctx, horizontal_wall.glob(), CollisionType::Impulse(impulse));
        ball.add_interaction(ctx, vertical_wall.glob(), CollisionType::Sensor);
        ball.add_interaction(ctx, paddle.glob(), CollisionType::Sensor);
        Self {
            horizontal_wall,
            vertical_wall,
            paddle,
            ball,
        }
    }
}
