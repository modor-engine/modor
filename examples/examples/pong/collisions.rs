use modor::{BuiltEntity, EntityBuilder};
use modor_physics::{CollisionGroup, CollisionType};
use modor_resources::ResKey;

pub(crate) const WALL_GROUP: ResKey<CollisionGroup> = ResKey::new("wall");
pub(crate) const PADDLE_GROUP: ResKey<CollisionGroup> = ResKey::new("paddle");
pub(crate) const BALL_GROUP: ResKey<CollisionGroup> = ResKey::new("ball");

pub(crate) fn collision_groups() -> impl BuiltEntity {
    EntityBuilder::new()
        .child_component(CollisionGroup::new(WALL_GROUP, wall_collision_type))
        .child_component(CollisionGroup::new(PADDLE_GROUP, paddle_collision_type))
        .child_component(CollisionGroup::new(BALL_GROUP, ball_collision_type))
}

fn wall_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == BALL_GROUP {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}

fn paddle_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == BALL_GROUP {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}

fn ball_collision_type(group_key: ResKey<CollisionGroup>) -> CollisionType {
    if group_key == WALL_GROUP || group_key == PADDLE_GROUP {
        CollisionType::Sensor
    } else {
        CollisionType::None
    }
}
