use instant::Instant;
use modor::{
    systems, App, BuiltEntity, Component, Entity, EntityBuilder, Filter, NoSystem, Query,
    SingletonComponent, With, World,
};
use modor_graphics::{
    instance_2d, instance_group_2d, window_target, Color, Material, ZIndex2D, WINDOW_CAMERA_2D,
};
use modor_math::Vec2;
use modor_physics::Transform2D;
use std::time::Duration;

const GRID_SIZE: usize = 150;
const REFRESH_PERIOD: Duration = Duration::from_millis(100);

pub fn main() {
    App::new()
        .with_entity(modor_graphics::module())
        .with_entity(window_target())
        .with_entity(alive_cell_instance_group())
        .with_entity(background())
        .with_entity(Grid::load())
        .run(modor_graphics::runner);
}

fn background() -> impl BuiltEntity {
    instance_2d(WINDOW_CAMERA_2D, None)
}

fn alive_cell_instance_group() -> impl BuiltEntity {
    instance_group_2d::<With<AliveCell>>(WINDOW_CAMERA_2D, None)
        .updated(|m: &mut Material| m.color = Color::BLACK)
}

fn alive_cell(x: usize, y: usize) -> impl BuiltEntity {
    EntityBuilder::new()
        .component(Transform2D::new())
        .with(|t| t.position = to_word_position(x, y))
        .with(|t| t.size = Vec2::ONE / GRID_SIZE as f32)
        .component(ZIndex2D::from(1))
        .component(AliveCell)
}

fn to_word_position(x: usize, y: usize) -> Vec2 {
    Vec2::new(
        (x as f32 + 0.5) / GRID_SIZE as f32 - 0.5,
        0.5 - (y as f32 + 0.5) / GRID_SIZE as f32,
    )
}

#[derive(SingletonComponent)]
struct Grid {
    are_cells_alive: Vec<Vec<bool>>,
    last_update: Instant,
}

#[systems]
impl Grid {
    fn load() -> Self {
        let mut are_cells_alive = vec![vec![false; GRID_SIZE]; GRID_SIZE];
        for (x, line) in include_str!("../res/game-of-life-grid").lines().enumerate() {
            for (y, character) in line.chars().enumerate() {
                if character == 'X' {
                    are_cells_alive[x][y] = true;
                }
            }
        }
        Self {
            are_cells_alive,
            last_update: Instant::now(),
        }
    }

    #[run]
    fn update(&mut self) {
        if self.last_update.elapsed() > REFRESH_PERIOD {
            self.last_update = Instant::now();
            let are_cells_alive = self.are_cells_alive.clone();
            for x in 0..GRID_SIZE {
                for y in 0..GRID_SIZE {
                    let neighbor_count = Self::neighbor_count(&are_cells_alive, x, y);
                    let is_cell_alive = &mut self.are_cells_alive[x][y];
                    if *is_cell_alive && !(neighbor_count == 2 || neighbor_count == 3) {
                        *is_cell_alive = false;
                    } else if !*is_cell_alive && neighbor_count == 3 {
                        *is_cell_alive = true;
                    }
                }
            }
        }
    }

    #[run_after_previous]
    fn update_display(
        &self,
        entity: Entity<'_>,
        mut cells: Query<'_, (&mut Transform2D, Entity<'_>, Filter<With<AliveCell>>)>,
        mut world: World<'_>,
    ) {
        let mut cells = cells.iter_mut();
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if self.are_cells_alive[x][y] {
                    if let Some((cell_transform, _, _)) = cells.next() {
                        cell_transform.position = to_word_position(x, y);
                    } else {
                        world.create_child_entity(entity.id(), alive_cell(x, y));
                    }
                }
            }
        }
        for (_, cell_entity, _) in cells {
            world.delete_entity(cell_entity.id());
        }
    }

    fn neighbor_count(are_cells_alive: &[Vec<bool>], x: usize, y: usize) -> u8 {
        let first_x = x.saturating_sub(1);
        let first_y = y.saturating_sub(1);
        let mut neighbor_count = 0;
        for (neighbor_x, line) in are_cells_alive.iter().enumerate().skip(first_x).take(3) {
            for (neighbor_y, &is_alive) in line.iter().enumerate().skip(first_y).take(3) {
                if (neighbor_x != x || neighbor_y != y) && is_alive {
                    neighbor_count += 1;
                }
            }
        }
        neighbor_count
    }
}

#[derive(Component, NoSystem)]
struct AliveCell;
