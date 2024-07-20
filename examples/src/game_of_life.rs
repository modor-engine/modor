use instant::Instant;
use modor::log::Level;
use modor::{Context, Node, RootNode, Visit};
use modor_graphics::{Color, Sprite2D};
use modor_physics::modor_math::Vec2;
use std::time::Duration;

const GRID_SIZE: usize = 150;
const REFRESH_PERIOD: Duration = Duration::from_millis(100);

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

#[derive(Visit)]
struct Root {
    last_update: Instant,
    background: Sprite2D,
    are_cells_alive: Vec<Vec<bool>>,
    cells: Vec<Sprite2D>,
}

impl Node for Root {
    fn on_enter(&mut self, ctx: &mut Context<'_>) {
        if self.last_update.elapsed() < REFRESH_PERIOD {
            return;
        }
        self.last_update = Instant::now();
        let alive_cell_count = self.refresh_grid();
        self.update_alive_cells(ctx, alive_cell_count);
    }
}

impl RootNode for Root {
    fn on_create(ctx: &mut Context<'_>) -> Self {
        let mut are_cells_alive = vec![vec![false; GRID_SIZE]; GRID_SIZE];
        for (x, line) in include_str!("../res/game-of-life-grid").lines().enumerate() {
            for (y, character) in line.chars().enumerate() {
                if character == 'X' {
                    are_cells_alive[x][y] = true;
                }
            }
        }
        Self {
            last_update: Instant::now(),
            background: Sprite2D::new(ctx, "background"),
            are_cells_alive,
            cells: vec![],
        }
    }
}

impl Root {
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

    fn cell_position(x: usize, y: usize) -> Vec2 {
        Vec2::new(
            (x as f32 + 0.5) / GRID_SIZE as f32 - 0.5,
            0.5 - (y as f32 + 0.5) / GRID_SIZE as f32,
        )
    }

    fn refresh_grid(&mut self) -> usize {
        let mut alive_cell_count = 0;
        let old_are_cells_alive = self.are_cells_alive.clone();
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                let neighbor_count = Self::neighbor_count(&old_are_cells_alive, x, y);
                let is_cell_alive = &mut self.are_cells_alive[x][y];
                if *is_cell_alive && !(neighbor_count == 2 || neighbor_count == 3) {
                    *is_cell_alive = false;
                } else if !*is_cell_alive && neighbor_count == 3 {
                    *is_cell_alive = true;
                }
                if *is_cell_alive {
                    alive_cell_count += 1;
                }
            }
        }
        alive_cell_count
    }

    fn update_alive_cells(&mut self, ctx: &mut Context<'_>, alive_cell_count: usize) {
        self.cells
            .resize_with(alive_cell_count, || Self::alive_cell(ctx));
        let mut current_cell_index = 0;
        for x in 0..GRID_SIZE {
            for y in 0..GRID_SIZE {
                if self.are_cells_alive[x][y] {
                    self.cells[current_cell_index].model.position = Self::cell_position(x, y);
                    current_cell_index += 1;
                }
            }
        }
    }

    fn alive_cell(ctx: &mut Context<'_>) -> Sprite2D {
        Sprite2D::new(ctx, "cell")
            .with_model(|m| m.size = Vec2::ONE / GRID_SIZE as f32)
            .with_model(|m| m.z_index = 1)
            .with_material(|m| m.color = Color::BLACK)
    }
}
