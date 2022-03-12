use modor::testing::TestApp;
use modor::{system, Built, Entity, EntityBuilder, EntityMainComponent, Query, SystemRunner, With};

#[derive(PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(PartialEq)]
struct Size {
    width: i32,
    height: i32,
}

impl Size {
    fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

struct Point;

impl Point {
    fn build(position: Position) -> impl Built<Self> {
        EntityBuilder::new(Self).with(position).with(Size {
            width: 0,
            height: 0,
        })
    }
}

impl EntityMainComponent for Point {
    type Type = ();
}

struct Object {
    is_collided_1: bool,
    is_collided_2: bool,
}

impl Object {
    fn build(position: Position, size: Size) -> impl Built<Self> {
        EntityBuilder::new(Self {
            is_collided_1: false,
            is_collided_2: false,
        })
        .with(position)
        .with(size)
    }

    fn detect_collisions_v1(
        mut objects: Query<'_, (&mut Self, &Position, &Size, Entity<'_>)>,
        other_objects: Query<'_, (&Position, &Size, Entity<'_>), With<Self>>,
    ) {
        for (object1, pos1, size1, entity1) in objects.iter_mut() {
            for (pos2, size2, entity2) in other_objects.iter() {
                if entity1.id() == entity2.id() {
                    continue;
                }
                if Self::is_inside(pos1, pos2, size1, size2)
                    || Self::is_inside(pos2, pos1, size2, size1)
                {
                    object1.is_collided_1 = true;
                }
            }
        }
    }

    fn detect_collisions_v2(
        &mut self,
        pos: &Position,
        size: &Size,
        entity: Entity<'_>,
        other_objects: Query<'_, (&Position, &Size, Entity<'_>), With<Self>>,
    ) {
        for (pos2, size2, entity2) in other_objects.iter() {
            if entity.id() == entity2.id() {
                continue;
            }
            if Self::is_inside(pos, pos2, size, size2) || Self::is_inside(pos2, pos, size2, size) {
                self.is_collided_2 = true;
            }
        }
    }

    fn is_inside(pos1: &Position, pos2: &Position, size1: &Size, size2: &Size) -> bool {
        let is_left_collision = pos1.x >= pos2.x && pos1.x < pos2.x + size2.width;
        let right = pos1.x + size1.width;
        let is_right_collision = right >= pos2.x && right < pos2.x + size2.width;
        let is_top_collision = pos1.y >= pos2.y && pos1.y < pos2.y + size2.height;
        let bottom = pos1.y + size1.height;
        let is_bottom_collision = bottom >= pos2.y && bottom < pos2.y + size2.height;
        (is_left_collision || is_right_collision) && (is_top_collision || is_bottom_collision)
    }
}

impl EntityMainComponent for Object {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
            .run(system!(Self::detect_collisions_v1))
            .run(system!(Self::detect_collisions_v2))
    }
}

#[test]
fn run_queries() {
    let mut app = TestApp::new();
    let object1_id = app.create_entity(Object::build(Position::new(0, 0), Size::new(1, 1)));
    let _ = app.create_entity(Point::build(Position::new(0, 0)));
    let object2_id = app.create_entity(Object::build(Position::new(5, 4), Size::new(2, 1)));
    let object3_id = app.create_entity(Object::build(Position::new(-1, -2), Size::new(3, 4)));
    app.update();
    app.assert_entity(object1_id)
        .has::<Object, _>(|c| assert!(c.is_collided_1))
        .has::<Object, _>(|c| assert!(c.is_collided_2));
    app.assert_entity(object2_id)
        .has::<Object, _>(|c| assert!(!c.is_collided_1))
        .has::<Object, _>(|c| assert!(!c.is_collided_2));
    app.assert_entity(object3_id)
        .has::<Object, _>(|c| assert!(c.is_collided_1))
        .has::<Object, _>(|c| assert!(c.is_collided_2));
}