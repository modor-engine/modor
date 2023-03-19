use crate::{AliveCell, GRID_WIDTH, RAW_SAVED_GRID, REFRESH_FREQUENCY, START_STOP_KEY};
use instant::Instant;
use modor::{systems, Entity, Query, Single, SingleMut, SingletonComponent, World};
use modor_input::Keyboard;

#[derive(SingletonComponent)]
pub(crate) struct Simulation {
    is_enabled: bool,
    last_update_time: Instant,
}

#[systems]
impl Simulation {
    pub(crate) fn new() -> Self {
        Self {
            is_enabled: false,
            last_update_time: Instant::now(),
        }
    }

    fn start(&mut self) -> bool {
        let now = Instant::now();
        if now - self.last_update_time > REFRESH_FREQUENCY && self.is_enabled {
            self.last_update_time = now;
            true
        } else {
            false
        }
    }

    #[run]
    fn update(&mut self, keyword: Single<'_, Keyboard>) {
        if keyword.key(START_STOP_KEY).is_just_released {
            self.is_enabled = !self.is_enabled;
        }
    }
}

#[derive(SingletonComponent, Clone)]
pub(crate) struct Grid {
    cells: Vec<CellState>,
}

#[systems]
impl Grid {
    pub(crate) fn new() -> Self {
        let mut grid = Self {
            cells: vec![CellState::Dead; GRID_WIDTH * GRID_WIDTH],
        };
        for (line_id, line) in RAW_SAVED_GRID.split('\n').enumerate() {
            for (column_id, column_value) in line.chars().enumerate() {
                if column_value == 'X' {
                    grid.set_cell_state(column_id, line_id, CellState::Alive);
                }
            }
        }
        grid
    }

    pub(crate) fn cell_state(&self, x: usize, y: usize) -> CellState {
        if x < GRID_WIDTH && y < GRID_WIDTH {
            self.cells[x * GRID_WIDTH + y]
        } else {
            CellState::Dead
        }
    }

    pub(crate) fn set_cell_state(&mut self, x: usize, y: usize, state: CellState) {
        if x * GRID_WIDTH + y < self.cells.len() {
            self.cells[x * GRID_WIDTH + y] = state;
        }
    }

    #[run]
    fn update(&mut self, mut simulation: SingleMut<'_, Simulation>) {
        if !simulation.start() {
            return;
        }
        let old_grid = self.clone();
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_WIDTH {
                let neighbor_count = old_grid.neighbor_count(x, y);
                let is_cell_alive = self.cell_state(x, y) == CellState::Alive;
                if is_cell_alive && !(neighbor_count == 2 || neighbor_count == 3) {
                    self.set_cell_state(x, y, CellState::Dead);
                } else if !is_cell_alive && neighbor_count == 3 {
                    self.set_cell_state(x, y, CellState::Alive);
                }
            }
        }
    }

    #[run_after_previous]
    fn update_view(
        &self,
        entity: Entity<'_>,
        mut alive_cells: Query<'_, (Entity<'_>, &mut AliveCell)>,
        mut world: World<'_>,
    ) {
        let mut alive_cells_iter = alive_cells.iter_mut();
        for (cell_id, &cell_state) in self.cells.iter().enumerate() {
            if cell_state == CellState::Alive {
                let (x, y) = Self::cell_id_to_position(cell_id);
                if let Some((_, next_cell)) = alive_cells_iter.next() {
                    next_cell.set_position(x, y);
                } else {
                    world.create_child_entity(entity.id(), AliveCell::build(x, y));
                }
            }
        }
        for (cell_entity, _) in alive_cells_iter {
            world.delete_entity(cell_entity.id());
        }
    }

    fn neighbor_count(&self, x: usize, y: usize) -> u8 {
        let mut neighbor_count = 0;
        for cell_x in x.saturating_sub(1)..(x + 2).min(GRID_WIDTH) {
            for cell_y in y.saturating_sub(1)..(y + 2).min(GRID_WIDTH) {
                let is_center = cell_x == x && cell_y == y;
                if !is_center && self.cell_state(cell_x, cell_y) == CellState::Alive {
                    neighbor_count += 1;
                }
            }
        }
        neighbor_count
    }

    #[allow(clippy::integer_division)]
    fn cell_id_to_position(id: usize) -> CellPosition {
        (id / GRID_WIDTH, id % GRID_WIDTH)
    }
}

pub(crate) type CellPosition = (usize, usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellState {
    Alive,
    Dead,
}

impl CellState {
    pub(crate) fn toggled(self) -> Self {
        match self {
            Self::Alive => Self::Dead,
            Self::Dead => Self::Alive,
        }
    }
}
