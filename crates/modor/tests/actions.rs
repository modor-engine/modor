#[macro_use]
extern crate modor;

use modor::testing::TestApp;
use modor::{Built, EntityBuilder, Query, With};

#[derive(Debug, PartialEq)]
struct Position(usize, usize);

struct Enemy;

#[entity]
impl Enemy {
    fn build(position: Position) -> impl Built<Self> {
        EntityBuilder::new(Self).with(position)
    }

    #[run_as(EnemyPositionUpdateAction)]
    fn update(position: &mut Position) {
        position.0 += 1;
        position.1 += 2;
    }
}

struct Selection(Position);

#[entity]
impl Selection {
    fn build(position: Position) -> impl Built<Self> {
        EntityBuilder::new(Self(position))
    }

    #[run_after(EnemyPositionUpdateAction)]
    fn update(&mut self, enemy_positions: Query<'_, &Position, With<Enemy>>) {
        if let Some(enemy_positions) = enemy_positions.iter().next() {
            self.0 .0 = enemy_positions.0;
            self.0 .1 = enemy_positions.1;
        }
    }
}

struct DisplayManager(usize, Vec<String>);

#[entity]
impl DisplayManager {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0, vec![]))
    }

    #[run_as(PositionDisplayAction)]
    fn print_positions(
        &mut self,
        enemy_positions: Query<'_, &Position, With<Enemy>>,
        selection_positions: Query<'_, &Selection>,
    ) {
        self.1.push(format!("Frame {}", self.0));
        for enemy_position in enemy_positions.iter() {
            self.1.push(format!("Enemy {:?}", enemy_position));
        }
        if let Some(selection) = selection_positions.iter().next() {
            self.1.push(format!("Selection {:?}", selection.0));
        }
    }

    #[run_after_previous]
    fn increment_frame_index(&mut self) {
        self.0 += 1;
    }
}

#[action]
struct PositionDisplayAction;

#[action(PositionDisplayAction)]
struct EnemyPositionUpdateAction;

#[test]
fn run_ordered_systems() {
    let mut app = TestApp::new();
    let enemy_id = app.create_entity(Enemy::build(Position(0, 0)));
    let display_manager_id = app.create_entity(DisplayManager::build());
    let selection_id = app.create_entity(Selection::build(Position(0, 0)));
    app.update();
    app.update();
    app.assert_entity(enemy_id)
        .has::<Position, _>(|c| assert_eq!(c, &Position(2, 4)))
        .has::<Enemy, _>(|_| ());
    app.assert_entity(selection_id)
        .has::<Selection, _>(|c| assert_eq!(c.0, Position(2, 4)));
    app.assert_entity(display_manager_id)
        .has::<DisplayManager, _>(|c| assert_eq!(c.0, 2))
        .has::<DisplayManager, _>(|c| {
            assert_eq!(
                c.1,
                vec![
                    "Frame 0",
                    "Enemy Position(0, 0)",
                    "Selection Position(0, 0)",
                    "Frame 1",
                    "Enemy Position(1, 2)",
                    "Selection Position(1, 2)"
                ]
            );
        });
}
