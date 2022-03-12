use modor::testing::TestApp;
use modor::{
    define_action, system, Built, DependsOn, EntityBuilder, EntityMainComponent, Query,
    SystemRunner, With,
};

#[derive(Debug, PartialEq)]
struct Position(usize, usize);

struct Enemy;

impl Enemy {
    fn build(position: Position) -> impl Built<Self> {
        EntityBuilder::new(Self).with(position)
    }

    fn update(position: &mut Position) {
        position.0 += 1;
        position.1 += 2;
    }
}

impl EntityMainComponent for Enemy {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run_as::<EnemyPositionUpdateAction>(system!(Self::update))
    }
}

struct Selection(Position);

impl Selection {
    fn build(position: Position) -> impl Built<Self> {
        EntityBuilder::new(Self(position))
    }

    fn update(&mut self, enemy_positions: Query<'_, &Position, With<Enemy>>) {
        if let Some(enemy_positions) = enemy_positions.iter().next() {
            self.0 .0 = enemy_positions.0;
            self.0 .1 = enemy_positions.1;
        }
    }
}

impl EntityMainComponent for Selection {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner.run_constrained::<DependsOn<EnemyPositionUpdateAction>>(system!(Self::update))
    }
}

struct DisplayManager(usize, Vec<String>);

impl DisplayManager {
    fn build() -> impl Built<Self> {
        EntityBuilder::new(Self(0, vec![]))
    }

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

    fn increment_frame_index(&mut self) {
        self.0 += 1;
    }
}

impl EntityMainComponent for DisplayManager {
    type Type = ();

    fn on_update(runner: SystemRunner<'_>) -> SystemRunner<'_> {
        runner
            .run_as::<PositionDisplayAction>(system!(Self::print_positions))
            .and_then(system!(Self::increment_frame_index))
    }
}

define_action!(PositionDisplayAction);
define_action!(EnemyPositionUpdateAction: PositionDisplayAction);

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
