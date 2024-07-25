use instant::Instant;
use modor::log::Level;
use modor::{App, FromApp, RootNode};
use modor_graphics::{Color, Sprite2D};
use modor_physics::modor_math::Vec2;
use std::time::Duration;

const GRID_SIZE: usize = 150;
const REFRESH_PERIOD: Duration = Duration::from_millis(100);

pub fn main() {
    modor_graphics::run::<Root>(Level::Info);
}

struct Root {
    last_update: Instant,
    background: Sprite2D,
    are_cells_alive: Vec<Vec<bool>>,
    cells: Vec<Sprite2D>,
}

impl FromApp for Root {
    fn from_app(app: &mut App) -> Self {
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
            background: Sprite2D::new(app),
            are_cells_alive,
            cells: vec![],
        }
    }
}

impl RootNode for Root {
    fn update(&mut self, app: &mut App) {
        if self.last_update.elapsed() < REFRESH_PERIOD {
            return;
        }
        self.last_update = Instant::now();
        let alive_cell_count = self.refresh_grid();
        self.update_alive_cells(app, alive_cell_count);
        self.background.update(app);
        for cell in &mut self.cells {
            cell.update(app);
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

    fn update_alive_cells(&mut self, app: &mut App, alive_cell_count: usize) {
        self.cells
            .resize_with(alive_cell_count, || Self::alive_cell(app));
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

    fn alive_cell(app: &mut App) -> Sprite2D {
        Sprite2D::new(app)
            .with_model(|m| m.size = Vec2::ONE / GRID_SIZE as f32)
            .with_model(|m| m.z_index = 1)
            .with_material(|m| m.color = Color::BLACK)
    }
}
