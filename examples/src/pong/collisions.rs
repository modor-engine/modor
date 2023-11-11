use modor::{BuiltEntity, EntityBuilder};
use modor_physics::{CollisionGroup, CollisionType, Impulse};
use modor_resources::ResKey;

pub(crate) const HORIZONTAL_WALL_GROUP: ResKey<CollisionGroup> = ResKey::new("horizontal-wall");
pub(crate) const LEFT_WALL_GROUP: ResKey<CollisionGroup> = ResKey::new("left-wall");
pub(crate) const RIGHT_WALL_GROUP: ResKey<CollisionGroup> = ResKey::new("right-wall");
pub(crate) const PADDLE_GROUP: ResKey<CollisionGroup> = ResKey::new("paddle");
pub(crate) const BALL_GROUP: ResKey<CollisionGroup> = ResKey::new("ball");

pub(crate) fn collision_groups() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(CollisionGroup::new(
            HORIZONTAL_WALL_GROUP,
            wall_collision_type,
        ))
        .child_component(CollisionGroup::new(LEFT_WALL_GROUP, wall_collision_type))
        .child_component(CollisionGroup::new(RIGHT_WALL_GROUP, wall_collision_type))
        .child_component(CollisionGroup::new(PADDLE_GROUP, paddle_collision_type))
        .child_component(CollisionGroup::new(BALL_GROUP, ball_collision_type))
}

fn wall_collision_type(_group_key: ResKey<CollisionGroup>) -> CollisionType {
    CollisionType::None
}

fn paddle_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == HORIZONTAL_WALL_GROUP {
        CollisionType::Impulse(Impulse::new(0., 0.))
    } else {
        CollisionType::None
    }
}

fn ball_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == HORIZONTAL_WALL_GROUP {
        CollisionType::Impulse(Impulse::new(1., 0.))
    } else if group_key == PADDLE_GROUP
        || group_key == LEFT_WALL_GROUP
        || group_key == RIGHT_WALL_GROUP
    {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}
