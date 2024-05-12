#![allow(missing_docs)]

use instant::Instant;
use modor::log::{info, Level};
use modor::{App, Context, Node, RootNode, Visit};
use modor_physics::modor_math::Vec2;
use modor_physics::{Body2D, CollisionGroup, CollisionType, Delta, Impulse};
use std::time::Duration;

fn main() {
    let start = Instant::now();
    let mut app = App::new::<Root>(Level::Info);
    info!("Build time: {:?}", start.elapsed());
    let start = Instant::now();
    let update_count = 100;
    for _ in 0..update_count {
        app.update();
    }
    info!("Update time: {:?}", start.elapsed() / update_count);
}

#[derive(Node, Visit)]
struct Root {
    bodies: Vec<BodyWrapper>,
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        ctx.root::<Delta>().get_mut(ctx).duration = Duration::from_secs(1);
        Self {
            bodies: (0..10_000).map(|i| BodyWrapper::new(ctx, i)).collect(),
        }
    }
}

#[derive(Node, Visit)]
struct BodyWrapper(Body2D);

impl BodyWrapper {
    #[allow(clippy::cast_precision_loss)]
    fn new(ctx: &mut Context<'_>, index: usize) -> Self {
        let mut body = Body2D::new(ctx, Vec2::ZERO, Vec2::ONE);
        body.position = Vec2::new(index as f32 * 0.5, index as f32 * 0.5) * 0.5;
        body.size = Vec2::ONE * 0.1;
        body.velocity = Vec2::new(1., 2.);
        body.mass = 1.;
        body.collision_group = Some(if index % 2 == 0 {
            ctx.root::<CollisionGroups>().get(ctx).group1.glob().clone()
        } else {
            ctx.root::<CollisionGroups>().get(ctx).group2.glob().clone()
        });
        Self(body)
    }
}

#[derive(Node, Visit)]
struct CollisionGroups {
    group1: CollisionGroup,
    group2: CollisionGroup,
}

impl RootNode for CollisionGroups {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let group1 = CollisionGroup::new(ctx);
        let group2 = CollisionGroup::new(ctx);
        group1.add_interaction(
            ctx,
            group2.glob(),
            CollisionType::Impulse(Impulse::default()),
        );
        Self { group1, group2 }
    }
}
